use std::collections::BTreeMap;

use move_binary_format::{
    errors::PartialVMResult,
    file_format::Bytecode,
    internals::ModuleIndex,
};
use move_vm_types::{
    values::{IntegerValue, StructRef, VectorRef, VMValueCast},
    views::ValueView,
};

use crate::{
    interpreter::{
        Frame,
        Interpreter
    },
    loader::Resolver,
};
use crate::witnessing::{BinaryIntegerOperationType, Footprint, Operation};
use crate::witnessing::traced_value::{Reference, ReferenceValueVisitor, TracedValue};

#[derive(Default, Clone)]
pub(crate) struct Footprints {
    state: FootprintState,
    pub data: Vec<Footprint>,
}

#[derive(Default, Clone)]
pub(crate) struct FootprintState {
    // frame_index -> (local_index -> addressing)
    local_value_addressings: BTreeMap<usize, BTreeMap<usize, BTreeMap<usize, Vec<usize>>>>,
    // raw_address -> (frame_index, local_index, sub_index)
    reverse_local_value_addressings: BTreeMap<usize, Reference>,
}

impl FootprintState {
    fn add_local(
        &mut self,
        frame_index: usize,
        local_index: usize,
        sub_indexes: BTreeMap<usize, Vec<usize>>,
    ) {
        let _ = self
            .local_value_addressings
            .entry(frame_index)
            .or_default()
            .insert(local_index, sub_indexes.clone())
            .unwrap();
        for (raw_address, sub_index) in sub_indexes {
            self.reverse_local_value_addressings
                .insert(
                    raw_address,
                    Reference::new(frame_index, local_index, sub_index),
                )
                .unwrap();
        }
    }

    fn remove_local(&mut self, frame_index: usize, local_index: usize) {
        self.local_value_addressings
            .entry(frame_index)
            .or_default()
            .remove(&local_index);
        // delete any in (frame_index, local_index)
        self.reverse_local_value_addressings
            .retain(|_k, v| !(v.frame_index == frame_index && v.local_index == local_index));
    }

    fn remove_locals(&mut self, frame_index: usize) {
        self.local_value_addressings.remove(&frame_index);
        self.reverse_local_value_addressings
            .retain(|_k, v| v.frame_index != frame_index);
    }
}

#[macro_export]
macro_rules! footprint {
    ($frame:expr, $instr:tt, $resolver:expr, $interp:expr) => {
        // only do footprint when the feature enabled
        $crate::interpreter::footprint::footprinting($frame, $instr, $resolver, $interp)
    };
}

pub(crate) fn footprinting(
    frame: &mut Frame,
    instr: &Bytecode,
    resolver: &Resolver,
    interp: &mut Interpreter,
) -> PartialVMResult<()> {
    let function_desc = &frame.function;
    let locals = &frame.locals;
    let pc = frame.pc;

    let frame_index = interp.call_stack.0.len();
    let module_id = function_desc.module_id().cloned();
    let function_index = function_desc.index();
    let stack_pointer = interp.operand_stack.value.len();

    let _caller_frame = interp.call_stack.0.last();
    let operation = match instr {
        Bytecode::Pop => {
            let val = interp.operand_stack.last_n(1)?.last().unwrap();
            Operation::Pop {
                poped_value: TracedValue::from(val).items(),
            }
        },
        Bytecode::Ret => Operation::Ret,
        Bytecode::BrTrue(_) => {
            let val = interp
                .operand_stack
                .last_n(1)?
                .last()
                .unwrap()
                .copy_value()
                .unwrap()
                .value_as()?;
            Operation::BrTrue { cond_val: val }
        },
        Bytecode::BrFalse(_) => {
            let val = interp
                .operand_stack
                .last_n(1)?
                .last()
                .unwrap()
                .copy_value()?
                .cast()?;
            Operation::BrTrue { cond_val: val }
        },
        Bytecode::Branch(_) => Operation::Branch,
        Bytecode::LdU8(_)
        | Bytecode::LdU64(_)
        | Bytecode::LdU128(_)
        | Bytecode::LdU16(_)
        | Bytecode::LdU32(_)
        | Bytecode::LdU256(_)
        | Bytecode::LdTrue
        | Bytecode::LdFalse => Operation::LdSimple,
        Bytecode::LdConst(_) => Operation::LdConst,

        Bytecode::CastU8 => {
            let val: IntegerValue = interp
                .operand_stack
                .last_n(1)?
                .last()
                .unwrap()
                .copy_value()?
                .value_as()?;
            Operation::CastU8 { origin: val }
        },
        Bytecode::CastU64 => {
            let val: IntegerValue = interp
                .operand_stack
                .last_n(1)?
                .last()
                .unwrap()
                .copy_value()?
                .value_as()?;
            Operation::CastU64 { origin: val }
        },
        Bytecode::CastU128 => {
            let val: IntegerValue = interp
                .operand_stack
                .last_n(1)?
                .last()
                .unwrap()
                .copy_value()?
                .value_as()?;
            Operation::CastU128 { origin: val }
        },
        Bytecode::CastU16 => {
            let val: IntegerValue = interp
                .operand_stack
                .last_n(1)?
                .last()
                .unwrap()
                .copy_value()?
                .value_as()?;
            Operation::CastU16 { origin: val }
        },
        Bytecode::CastU32 => {
            let val: IntegerValue = interp
                .operand_stack
                .last_n(1)?
                .last()
                .unwrap()
                .copy_value()?
                .value_as()?;
            Operation::CastU32 { origin: val }
        },
        Bytecode::CastU256 => {
            let val: IntegerValue = interp
                .operand_stack
                .last_n(1)?
                .last()
                .unwrap()
                .copy_value()?
                .value_as()?;
            Operation::CastU256 { origin: val }
        },
        Bytecode::CopyLoc(idx) => {
            let local = locals.copy_loc(*idx as usize)?;
            Operation::CopyLoc {
                local: TracedValue::from(&local).items(),
            }
        },
        Bytecode::MoveLoc(idx) => {
            interp
                .footprints
                .state
                .remove_local(frame_index, *idx as usize);
            let local = locals.copy_loc(*idx as usize)?;
            Operation::MoveLoc {
                local: TracedValue::from(&local).items(),
            }
        },
        Bytecode::StLoc(idx) => {
            interp
                .footprints
                .state
                .remove_local(frame_index, *idx as usize);

            let new_value = interp.operand_stack.last_n(1)?.last().unwrap();
            let new_value: TracedValue = new_value.into();

            // value stored to loc only have 1 reference on it
            // so we can hook here to index every sub items by it rc-ptr.
            interp.footprints.state.add_local(
                frame_index,
                *idx as usize,
                new_value.container_sub_indexes(),
            );

            let old_local = locals.copy_loc(*idx as usize)?;
            Operation::StLoc {
                old_local: TracedValue::from(&old_local).items(),
                new_value: new_value.items(),
            }
        },
        Bytecode::Call(fh_idx) => {
            let func = resolver.function_from_handle(*fh_idx)?;
            Operation::Call {
                args: interp
                    .operand_stack
                    .last_n(func.param_count())?
                    .map(|t| TracedValue::from(t).items())
                    .collect::<Vec<_>>(),
            }
        },
        Bytecode::CallGeneric(fh_idx) => {
            let func = resolver.function_from_instantiation(*fh_idx)?;

            Operation::CallGeneric {
                args: interp
                    .operand_stack
                    .last_n(func.param_count())?
                    .map(|t| TracedValue::from(t).items())
                    .collect::<Vec<_>>(),
            }
        },
        Bytecode::Pack(sd_idx) => {
            let field_count = resolver.field_count(*sd_idx);

            Operation::Pack {
                args: interp
                    .operand_stack
                    .last_n(field_count as usize)?
                    .map(|t| TracedValue::from(t).items())
                    .collect::<Vec<_>>(),
            }
        },
        Bytecode::PackGeneric(si_idx) => {
            let field_count = resolver.field_instantiation_count(*si_idx);
            Operation::PackGeneric {
                args: interp
                    .operand_stack
                    .last_n(field_count as usize)?
                    .map(|t| TracedValue::from(t).items())
                    .collect::<Vec<_>>(),
            }
        },
        Bytecode::Unpack(_sd_idx) => Operation::Unpack {
            arg: interp
                .operand_stack
                .last_n(1)?
                .last()
                .map(|t| TracedValue::from(t).items())
                .unwrap(),
        },
        Bytecode::UnpackGeneric(_sd_idx) => Operation::UnpackGeneric {
            arg: interp
                .operand_stack
                .last_n(1)?
                .last()
                .map(|t| TracedValue::from(t).items())
                .unwrap(),
        },
        Bytecode::ReadRef => {
            let reference = interp
                .operand_stack
                .last_n(1)?
                .last()
                .unwrap()
                .copy_value()?;
            let mut visitor = ReferenceValueVisitor::default();
            reference.visit(&mut visitor);
            let pointer = visitor.reference_pointer;
            let value = reference
                .value_as::<move_vm_types::values::Reference>()?
                .read_ref()?;
            Operation::ReadRef {
                reference: interp
                    .footprints
                    .state
                    .reverse_local_value_addressings
                    .get(&pointer)
                    .cloned()
                    .unwrap(),
                value: TracedValue::from(&value).items(),
            }
        },
        Bytecode::WriteRef => {
            let reference = interp
                .operand_stack
                .last_n(1)?
                .last()
                .unwrap()
                .copy_value()?;
            let mut visitor = ReferenceValueVisitor::default();
            reference.visit(&mut visitor);
            let pointer = visitor.reference_pointer;
            let old_value = reference
                .value_as::<move_vm_types::values::Reference>()?
                .read_ref()?;
            let new_value = interp
                .operand_stack
                .last_n(2)?
                .next()
                .unwrap()
                .copy_value()?;
            Operation::WriteRef {
                reference: interp
                    .footprints
                    .state
                    .reverse_local_value_addressings
                    .get(&pointer)
                    .cloned()
                    .unwrap(),
                old_value: TracedValue::from(&old_value).items(),
                new_value: TracedValue::from(&new_value).items(),
            }
        },
        Bytecode::FreezeRef => Operation::FreezeRef,
        Bytecode::MutBorrowLoc(_idx) => Operation::BorrowLoc {
            imm: false,
            // reference: Reference::new(frame_index, *idx as usize, vec![0]),
        },
        Bytecode::ImmBorrowLoc(_idx) => Operation::BorrowLoc {
            imm: true,
            // not need, as outside can build the reference themselves
            // reference: Reference::new(frame_index, *idx as usize, vec![0]), // TODO: should add 0 or not
        },
        Bytecode::MutBorrowField(fh_idx) => {
            let reference: StructRef = interp
                .operand_stack
                .last_n(1)?
                .last()
                .unwrap()
                .copy_value()?
                .cast()?;
            let addr = reference.raw_address();
            let reference = interp
                .footprints
                .state
                .reverse_local_value_addressings
                .get(&addr)
                .cloned()
                .expect("index by ptr ok");
            let offset = resolver.field_offset(*fh_idx);
            Operation::BorrowField {
                imm: false,
                reference,
                field_offset: offset,
            }
        },
        Bytecode::MutBorrowFieldGeneric(fi_idx) => {
            let reference: StructRef = interp
                .operand_stack
                .last_n(1)?
                .last()
                .unwrap()
                .copy_value()?
                .cast()?;
            let addr = reference.raw_address();
            let reference = interp
                .footprints
                .state
                .reverse_local_value_addressings
                .get(&addr)
                .cloned()
                .expect("index by ptr ok");
            let offset = resolver.field_instantiation_offset(*fi_idx);
            Operation::BorrowFieldGeneric {
                imm: false,
                reference,
                field_offset: offset,
            }
        },
        Bytecode::ImmBorrowField(fh_idx) => {
            let reference: StructRef = interp
                .operand_stack
                .last_n(1)?
                .last()
                .unwrap()
                .copy_value()?
                .cast()?;
            let addr = reference.raw_address();

            let offset = resolver.field_offset(*fh_idx);
            Operation::BorrowField {
                imm: true,
                reference: interp
                    .footprints
                    .state
                    .reverse_local_value_addressings
                    .get(&addr)
                    .cloned()
                    .expect("index by ptr ok"),
                field_offset: offset,
            }
        },
        Bytecode::ImmBorrowFieldGeneric(fi_idx) => {
            let reference: StructRef = interp
                .operand_stack
                .last_n(1)?
                .last()
                .unwrap()
                .copy_value()?
                .cast()?;
            let addr = reference.raw_address();
            let reference = interp
                .footprints
                .state
                .reverse_local_value_addressings
                .get(&addr)
                .cloned()
                .expect("index by ptr ok");
            let offset = resolver.field_instantiation_offset(*fi_idx);
            Operation::BorrowFieldGeneric {
                imm: true,
                reference,
                field_offset: offset,
            }
        },
        Bytecode::Add
        | Bytecode::Sub
        | Bytecode::Mul
        | Bytecode::Mod
        | Bytecode::Div
        | Bytecode::BitOr
        | Bytecode::BitAnd
        | Bytecode::Xor
        | Bytecode::Lt
        | Bytecode::Gt
        | Bytecode::Le
        | Bytecode::Ge => {
            let mut operands = interp
                .operand_stack
                .last_n(2)?
                .map(|v| v.copy_value().and_then(|v| v.value_as::<IntegerValue>()))
                .collect::<PartialVMResult<Vec<_>>>()?;
            Operation::BinaryOp {
                ty: BinaryIntegerOperationType::try_from(instr.clone()).unwrap(),
                rhs: operands.pop().unwrap(),
                lhs: operands.pop().unwrap(),
            }
        },

        Bytecode::Or => {
            let mut operands = interp
                .operand_stack
                .last_n(2)?
                .map(|v| v.copy_value().and_then(|v| v.value_as::<bool>()))
                .collect::<PartialVMResult<Vec<_>>>()?;
            Operation::Or {
                rhs: operands.pop().unwrap(),
                lhs: operands.pop().unwrap(),
            }
        },
        Bytecode::And => {
            let mut operands = interp
                .operand_stack
                .last_n(2)?
                .map(|v| v.copy_value().and_then(|v| v.value_as::<bool>()))
                .collect::<PartialVMResult<Vec<_>>>()?;
            Operation::And {
                rhs: operands.pop().unwrap(),
                lhs: operands.pop().unwrap(),
            }
        },
        Bytecode::Not => {
            let mut operands = interp
                .operand_stack
                .last_n(1)?
                .map(|v| v.copy_value().and_then(|v| v.value_as::<bool>()))
                .collect::<PartialVMResult<Vec<_>>>()?;
            Operation::Not {
                value: operands.pop().unwrap(),
            }
        },
        Bytecode::Eq => {
            let mut operands = interp
                .operand_stack
                .last_n(2)?
                .map(|v| v.copy_value())
                .collect::<PartialVMResult<Vec<_>>>()?;
            Operation::Eq {
                rhs: TracedValue::from(&operands.pop().unwrap()).items(),
                lhs: TracedValue::from(&operands.pop().unwrap()).items(),
            }
        },
        Bytecode::Neq => {
            let mut operands = interp
                .operand_stack
                .last_n(2)?
                .map(|v| v.copy_value())
                .collect::<PartialVMResult<Vec<_>>>()?;
            Operation::Neq {
                rhs: TracedValue::from(&operands.pop().unwrap()).items(),
                lhs: TracedValue::from(&operands.pop().unwrap()).items(),
            }
        },

        Bytecode::Abort => {
            let value = interp
                .operand_stack
                .last_n(1)?
                .map(|v| v.copy_value())
                .collect::<PartialVMResult<Vec<_>>>()?
                .pop()
                .unwrap();
            Operation::Abort {
                error_code: value.value_as()?,
            }
        },
        Bytecode::Nop => Operation::Nop,
        Bytecode::Shl => {
            let mut operands = interp
                .operand_stack
                .last_n(2)?
                .map(|v| v.copy_value())
                .collect::<PartialVMResult<Vec<_>>>()?;
            Operation::Shl {
                rhs: operands.pop().unwrap().value_as()?,
                lhs: operands.pop().unwrap().value_as()?,
            }
        },
        Bytecode::Shr => {
            let mut operands = interp
                .operand_stack
                .last_n(2)?
                .map(|v| v.copy_value())
                .collect::<PartialVMResult<Vec<_>>>()?;
            Operation::Shr {
                rhs: operands.pop().unwrap().value_as()?,
                lhs: operands.pop().unwrap().value_as()?,
            }
        },
        Bytecode::VecPack(_si, num) => Operation::VecPack {
            args: interp
                .operand_stack
                .last_n(*num as usize)?
                .map(|t| TracedValue::from(t).items())
                .collect::<Vec<_>>(),
        },
        Bytecode::VecUnpack(_, _num) => Operation::VecUnpack {
            arg: interp
                .operand_stack
                .last_n(1)?
                .last()
                .map(|t| TracedValue::from(t).items())
                .unwrap(),
        },
        Bytecode::VecLen(si) => {
            let vec_ref = interp
                .operand_stack
                .last_n(1)?
                .map(|v| v.copy_value())
                .collect::<PartialVMResult<Vec<_>>>()?
                .pop()
                .unwrap();
            let mut reference_visitor = ReferenceValueVisitor::default();
            vec_ref.visit(&mut reference_visitor);
            assert!(reference_visitor.indexed.is_none());

            let vec_ref = vec_ref.value_as::<VectorRef>()?;

            let len = {
                let (ty, _ty_count) =
                    frame
                        .ty_cache
                        .get_signature_index_type(*si, resolver, &frame.ty_args)?;
                vec_ref.len(ty)?
            };
            Operation::VecLen {
                vec_ref: interp
                    .footprints
                    .state
                    .reverse_local_value_addressings
                    .get(&reference_visitor.reference_pointer)
                    .cloned()
                    .unwrap(),
                len: len.value_as()?,
            }
        },
        Bytecode::VecImmBorrow(si) | Bytecode::VecMutBorrow(si) => {
            let mut values = interp
                .operand_stack
                .last_n(2)?
                .map(|v| v.copy_value())
                .collect::<PartialVMResult<Vec<_>>>()?;
            let idx: u64 = values.pop().unwrap().value_as()?;
            let vec_ref = values.pop().unwrap();
            let mut reference_visitor = ReferenceValueVisitor::default();
            vec_ref.visit(&mut reference_visitor);
            assert!(reference_visitor.indexed.is_none());

            let vec_ref = vec_ref.value_as::<VectorRef>()?;
            let elem = {
                let (ty, _ty_count) =
                    frame
                        .ty_cache
                        .get_signature_index_type(*si, resolver, &frame.ty_args)?;
                vec_ref.borrow_elem(idx as usize, ty)?
            };

            Operation::VecBorrow {
                imm: matches!(instr, Bytecode::VecImmBorrow(_)),
                idx,
                vec_ref: interp
                    .footprints
                    .state
                    .reverse_local_value_addressings
                    .get(&reference_visitor.reference_pointer)
                    .cloned()
                    .unwrap(),

                elem: TracedValue::from(&elem).items(),
            }
        },
        Bytecode::VecPushBack(si) => {
            let mut values = interp
                .operand_stack
                .last_n(2)?
                .map(|v| v.copy_value())
                .collect::<PartialVMResult<Vec<_>>>()?;
            let elem = values.pop().unwrap();
            let vec_ref = values.pop().unwrap();

            let mut reference_visitor = ReferenceValueVisitor::default();
            vec_ref.visit(&mut reference_visitor);
            assert!(reference_visitor.indexed.is_none());

            let vec_ref = vec_ref.value_as::<VectorRef>()?;
            let (ty, _ty_count) =
                frame
                    .ty_cache
                    .get_signature_index_type(*si, resolver, &frame.ty_args)?;

            let vec_len = vec_ref.len(ty)?;

            Operation::VecPushBack {
                vec_ref: interp
                    .footprints
                    .state
                    .reverse_local_value_addressings
                    .get(&reference_visitor.reference_pointer)
                    .cloned()
                    .unwrap(),

                elem: TracedValue::from(&elem).items(),
                vec_len: vec_len.value_as()?,
            }
        },
        Bytecode::VecPopBack(si) => {
            let mut values = interp
                .operand_stack
                .last_n(1)?
                .map(|v| v.copy_value())
                .collect::<PartialVMResult<Vec<_>>>()?;
            let vec_ref = values.pop().unwrap();
            let mut reference_visitor = ReferenceValueVisitor::default();
            vec_ref.visit(&mut reference_visitor);
            assert!(reference_visitor.indexed.is_none());

            let vec_ref = vec_ref.value_as::<VectorRef>()?;
            let (ty, _ty_count) =
                frame
                    .ty_cache
                    .get_signature_index_type(*si, resolver, &frame.ty_args)?;

            let vec_len: u64 = vec_ref.len(ty)?.value_as()?;

            let elem = vec_ref.borrow_elem((vec_len - 1) as usize, ty)?;

            Operation::VecPopBack {
                vec_len,
                vec_ref: interp
                    .footprints
                    .state
                    .reverse_local_value_addressings
                    .get(&reference_visitor.reference_pointer)
                    .cloned()
                    .unwrap(),

                elem: TracedValue::from(&elem).items(),
            }
        },

        Bytecode::VecSwap(si) => {
            let mut values = interp
                .operand_stack
                .last_n(3)?
                .map(|v| v.copy_value())
                .collect::<PartialVMResult<Vec<_>>>()?;
            let idx2: u64 = values.pop().unwrap().value_as()?;
            let idx1: u64 = values.pop().unwrap().value_as()?;
            let vec_ref = values.pop().unwrap();
            let mut reference_visitor = ReferenceValueVisitor::default();
            vec_ref.visit(&mut reference_visitor);
            assert!(reference_visitor.indexed.is_none());

            let vec_ref = vec_ref.value_as::<VectorRef>()?;
            let (ty, _ty_count) =
                frame
                    .ty_cache
                    .get_signature_index_type(*si, resolver, &frame.ty_args)?;
            let vec_len: u64 = vec_ref.len(ty)?.value_as()?;
            let idx2_elem = vec_ref.borrow_elem(idx2 as usize, ty)?;
            let idx1_elem = vec_ref.borrow_elem(idx2 as usize, ty)?;

            Operation::VecSwap {
                vec_len,
                vec_ref: interp
                    .footprints
                    .state
                    .reverse_local_value_addressings
                    .get(&reference_visitor.reference_pointer)
                    .cloned()
                    .unwrap(),
                idx2,
                idx1,
                idx2_elem: TracedValue::from(&idx2_elem).items(),
                idx1_elem: TracedValue::from(&idx1_elem).items(),
            }
        },
        Bytecode::MutBorrowGlobal(_)
        | Bytecode::MutBorrowGlobalGeneric(_)
        | Bytecode::Exists(_)
        | Bytecode::ExistsGeneric(_)
        | Bytecode::MoveFrom(_)
        | Bytecode::MoveFromGeneric(_)
        | Bytecode::MoveTo(_)
        | Bytecode::MoveToGeneric(_)
        | Bytecode::ImmBorrowGlobal(_)
        | Bytecode::ImmBorrowGlobalGeneric(_) => {
            unimplemented!("unsupported instruction")
        },
    };

    interp.footprints.data.push(Footprint {
        op: instr.clone(),
        module_id,
        function_id: function_index.into_index(),
        pc,
        frame_index,
        stack_pointer,
        data: operation,
    });
    Ok(())
}
