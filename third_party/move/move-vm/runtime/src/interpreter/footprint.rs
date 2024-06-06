use std::collections::BTreeMap;

use move_binary_format::errors::PartialVMResult;
use move_binary_format::file_format::Bytecode;
use move_binary_format::internals::ModuleIndex;
use move_core_types::language_storage::ModuleId;
use move_vm_types::values::{IntegerValue, Locals, StructRef, VMValueCast};

use crate::interpreter::Interpreter;
use crate::interpreter::traced_value::{Reference, TracedValue, ValueItems};
use crate::loader::{Function, Resolver};

#[derive(Clone, Debug)]
pub enum Operation {
    Pop {
        poped_value: ValueItems,
    },
    Ret,
    BrTrue {
        cond_val: bool,
    },
    BrFalse {
        cond_val: bool,
    },
    Branch,
    LdSimple, // LdU{8,16,32,64,128,256}
    CopyLoc {
        local: ValueItems,
    },
    MoveLoc {
        local: ValueItems,
    },
    StLoc {
        old_local: ValueItems,
        new_value: ValueItems,
    },
    Call {
        args: Vec<ValueItems>,
    },
    BorrowLoc { imm: bool, reference: Reference },
    ImmBorrowLoc,
    BorrowField {
        imm: bool,
        reference: Reference,
        fd_index: usize,
    },
    CastU8 { origin: IntegerValue },
    CastU16 { origin: IntegerValue },
    CastU32 { origin: IntegerValue },
    CastU64 { origin: IntegerValue },
    CastU128 { origin: IntegerValue },
    CastU256 { origin: IntegerValue },
}

#[derive(Clone, Debug)]
pub struct Footprint {
    module_id: Option<ModuleId>,
    function_id: usize,
    pc: u16,
    frame_index: usize,
    stack_pointer: usize,
    op: Bytecode,
    data: Operation,

}
#[derive(Default)]
pub struct Footprints {
    state: FootprintState,
    data: Vec<Footprint>,
}

#[derive(Default)]
pub struct FootprintState {
    // frame_index -> (local_index -> addressing)
    local_value_addressings: BTreeMap<usize, BTreeMap<usize, BTreeMap<usize, Vec<usize>>>>,
    // raw_address -> (frame_index, local_index, sub_index)
    reverse_local_value_addressings: BTreeMap<usize, (usize, usize, Vec<usize>)>,
}

impl FootprintState {
    fn add_local(&mut self, frame_index: usize, local_index: usize, sub_indexes: BTreeMap<usize, Vec<usize>>) {
        let _ = self.local_value_addressings.entry(frame_index).or_default().insert(local_index, sub_indexes.clone()).unwrap();
        for (raw_address, sub_index) in sub_indexes {
            self.reverse_local_value_addressings.insert(raw_address, (frame_index, local_index, sub_index)).unwrap();
        }
    }
    fn remove_local(&mut self, frame_index: usize, local_index: usize) {
        self.local_value_addressings.entry(frame_index).or_default().remove(&local_index);
        // delete any in (frame_index, local_index)
        self.reverse_local_value_addressings.retain(|_k, v| !(v.0 == frame_index && v.1 == local_index));
    }
    fn remove_locals(&mut self, frame_index: usize) {
        self.local_value_addressings.remove(&frame_index);
        self.reverse_local_value_addressings.retain(|_k, v| !(v.0 == frame_index));
    }
}

#[macro_export]
macro_rules! footprint {
    ($function_desc:expr, $locals:expr, $pc:expr, $instr:tt, $resolver:expr, $interp:expr) => {
        // Only include this code in debug releases
        $crate::interpreter::footprint::footprinting(
            $function_desc,
            $locals,
            $pc,
            $instr,
            $resolver,
            $interp,
        )
    };
}

pub fn footprinting(
    function_desc: &Function,
    locals: &Locals,
    pc: u16,
    instr: &Bytecode,
    resolver: &Resolver,
    interp: &mut Interpreter,
) -> PartialVMResult<()> {
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
        }
        Bytecode::Ret => Operation::Ret,
        Bytecode::BrTrue(_) => {
            let val = interp
                .operand_stack
                .last_n(1)?
                .last()
                .unwrap().copy_value().unwrap().value_as()?;
            Operation::BrTrue { cond_val: val }
        }
        Bytecode::BrFalse(_) => {
            let val = interp
                .operand_stack
                .last_n(1)?
                .last()
                .unwrap().copy_value()?.cast()?;
            Operation::BrTrue { cond_val: val }
        }
        Bytecode::Branch(_) => Operation::Branch,
        Bytecode::LdU8(_) |
        Bytecode::LdU64(_) |
        Bytecode::LdU128(_) |
        Bytecode::LdU16(_) |
        Bytecode::LdU32(_) |
        Bytecode::LdU256(_) |
        Bytecode::LdTrue |
        Bytecode::LdFalse => Operation::LdSimple,
        Bytecode::LdConst(_) => { unimplemented!() }

        Bytecode::CastU8 => {
            let val: IntegerValue = interp
                .operand_stack
                .last_n(1)?
                .last()
                .unwrap().copy_value()?.value_as()?;
            Operation::CastU8 { origin: val }
        }
        Bytecode::CastU64 => {
            let val: IntegerValue = interp
                .operand_stack
                .last_n(1)?
                .last()
                .unwrap().copy_value()?.value_as()?;
            Operation::CastU64 { origin: val }
        }
        Bytecode::CastU128 => {
            let val: IntegerValue = interp
                .operand_stack
                .last_n(1)?
                .last()
                .unwrap().copy_value()?.value_as()?;
            Operation::CastU128 { origin: val }
        }
        Bytecode::CastU16 => {
            let val: IntegerValue = interp
                .operand_stack
                .last_n(1)?
                .last()
                .unwrap().copy_value()?.value_as()?;
            Operation::CastU16 { origin: val }
        }
        Bytecode::CastU32 => {
            let val: IntegerValue = interp
                .operand_stack
                .last_n(1)?
                .last()
                .unwrap().copy_value()?.value_as()?;
            Operation::CastU32 { origin: val }
        }
        Bytecode::CastU256 => {
            let val: IntegerValue = interp
                .operand_stack
                .last_n(1)?
                .last()
                .unwrap().copy_value()?.value_as()?;
            Operation::CastU256 { origin: val }
        }
        Bytecode::CopyLoc(idx) => {
            let local = locals.copy_loc(*idx as usize)?;
            Operation::CopyLoc {
                local: TracedValue::from(&local).items(),
            }
        }
        Bytecode::MoveLoc(idx) => {
            interp.footprints.state.remove_local(frame_index, *idx as usize);
            let local = locals.copy_loc(*idx as usize)?;
            Operation::MoveLoc {
                local: TracedValue::from(&local).items(),
            }
        }
        Bytecode::StLoc(idx) => {
            interp.footprints.state.remove_local(frame_index, *idx as usize);

            let new_value = interp.operand_stack.last_n(1)?.last().unwrap();
            let new_value: TracedValue = new_value.into();

            // value stored to loc only have 1 reference on it
            // so we can hook here to index every sub items by it rc-ptr.
            interp.footprints.state.add_local(frame_index, *idx as usize, new_value.container_sub_indexes());

            let old_local = locals.copy_loc(*idx as usize)?;
            Operation::StLoc {
                old_local: TracedValue::from(&old_local).items(),
                new_value: new_value.items(),
            }
        }
        Bytecode::Call(fh_idx) => {
            let func = resolver
                .function_from_handle(*fh_idx)?;
            Operation::Call {
                args: interp
                    .operand_stack
                    .last_n(func.param_count())?
                    .map(|t| TracedValue::from(t).items())
                    .collect::<Vec<_>>(),
            }
        }
        Bytecode::CallGeneric(_) => {
            todo!()
        }
        Bytecode::Pack(_) => {
            todo!()
        }
        Bytecode::PackGeneric(_) => {
            todo!()
        }
        Bytecode::Unpack(_) => {
            todo!()
        }
        Bytecode::UnpackGeneric(_) => {
            todo!()
        }
        Bytecode::ReadRef => {
            todo!()
        }
        Bytecode::WriteRef => {
            todo!()
        }
        Bytecode::FreezeRef => {
            todo!()
        }
        Bytecode::MutBorrowLoc(idx) => {
            Operation::BorrowLoc { imm: false, reference: Reference(frame_index, *idx as usize, vec![0]) }
        },
        Bytecode::ImmBorrowLoc(idx) => {
            Operation::BorrowLoc { imm: true, reference: Reference(frame_index, *idx as usize, vec![0]) }
        },
        Bytecode::MutBorrowField(fh_idx) => {
            let reference: StructRef = interp.operand_stack.last_n(1)?.last().unwrap().copy_value()?.cast()?;
            let addr = reference.raw_address();
            let (frame_index, local_index, sub_index) = interp.footprints.state.reverse_local_value_addressings.get(&addr).cloned().expect("index by ptr ok");
            let offset = resolver.field_offset(*fh_idx);
            Operation::BorrowField { imm: false, reference: Reference(frame_index, local_index, sub_index), fd_index: offset }
        }
        Bytecode::MutBorrowFieldGeneric(_) => {
            todo!()
        }
        Bytecode::ImmBorrowField(fh_idx) => {
            let reference: StructRef = interp.operand_stack.last_n(1)?.last().unwrap().copy_value()?.cast()?;
            let addr = reference.raw_address();
            let (frame_index, local_index, sub_index) = interp.footprints.state.reverse_local_value_addressings.get(&addr).cloned().expect("index by ptr ok");
            let offset = resolver.field_offset(*fh_idx);
            Operation::BorrowField { imm: true, reference: Reference(frame_index, local_index, sub_index), fd_index: offset }
        }
        Bytecode::ImmBorrowFieldGeneric(_) => {
            todo!()
        }
        Bytecode::MutBorrowGlobal(_) => {
            unimplemented!()
        }
        Bytecode::MutBorrowGlobalGeneric(_) => {
            unimplemented!()
        }
        Bytecode::Add => {
            todo!()
        }
        Bytecode::Sub => {
            todo!()
        }
        Bytecode::Mul => {
            todo!()
        }
        Bytecode::Mod => {
            todo!()
        }
        Bytecode::Div => {
            todo!()
        }
        Bytecode::BitOr => { todo!() }
        Bytecode::BitAnd => { todo!() }
        Bytecode::Xor => { todo!() }
        Bytecode::Or => { todo!() }
        Bytecode::And => { todo!() }
        Bytecode::Not => { todo!() }
        Bytecode::Eq => { todo!() }
        Bytecode::Neq => { todo!() }
        Bytecode::Lt => { todo!() }
        Bytecode::Gt => { todo!() }
        Bytecode::Le => { todo!() }
        Bytecode::Ge => { todo!() }
        Bytecode::Abort => { todo!() }
        Bytecode::Nop => { todo!() }
        Bytecode::Shl => { todo!() }
        Bytecode::Shr => { todo!() }
        Bytecode::VecPack(_, _) => { todo!() }
        Bytecode::VecLen(_) => { todo!() }
        Bytecode::VecImmBorrow(_) => { todo!() }
        Bytecode::VecMutBorrow(_) => { todo!() }
        Bytecode::VecPushBack(_) => { todo!() }
        Bytecode::VecPopBack(_) => { todo!() }
        Bytecode::VecUnpack(_, _) => { todo!() }
        Bytecode::VecSwap(_) => { todo!() }
        Bytecode::ImmBorrowGlobal(_) => { unimplemented!() }
        Bytecode::ImmBorrowGlobalGeneric(_) => { unimplemented!() }
        Bytecode::Exists(_) => { unimplemented!() }
        Bytecode::ExistsGeneric(_) => { unimplemented!() }
        Bytecode::MoveFrom(_) => { unimplemented!() }
        Bytecode::MoveFromGeneric(_) => { unimplemented!() }
        Bytecode::MoveTo(_) => { unimplemented!() }
        Bytecode::MoveToGeneric(_) => { unimplemented!() }
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
