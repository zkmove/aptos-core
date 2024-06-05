use std::collections::BTreeMap;

use crate::interpreter::traced_value::TracedValue;

pub enum Footprint {
    Pop {
        poped_value: TracedValue,
    },
    Ret,
    BrTrue {
        cond_val: bool,
    },
    BrFalse {
        cond_val: bool,
    },
    Branch,
    Ld, // LdU{8,16,32,64,128,256}
    CopyLoc {
        local: TracedValue,
    },
    MoveLoc {
        local: TracedValue,
    },
    StLoc {
        old_local: TracedValue,
        new_value: TracedValue,
    },
    Call {
        args: Vec<TracedValue>,
    },
    MutBorrowLoc,
    ImmBorrowLoc,
}

pub struct Call {}

#[derive(Default)]
pub struct Footprints {
    state: FootprintState,
    data: Vec<Footprint>,
}

#[derive(Default)]
pub struct FootprintState {
    pub(crate) container_addressing: BTreeMap<usize, Vec<usize>>,
}
// #[macro_export]
// macro_rules! footprint {
//     ($function_desc:expr, $locals:expr, $pc:expr, $instr:tt, $resolver:expr, $interp:expr) => {
//         // Only include this code in debug releases
//         $crate::interpreter::footprint:footprinting(
//             &$function_desc,
//             $locals,
//             $pc,
//             &$instr,
//             $resolver,
//             $interp,
//         )
//     };
// }

// pub fn footprinting(
//     function_desc: &Function,
//     locals: &Locals,
//     pc: u16,
//     instr: &Bytecode,
//     resolver: &Resolver,
//     interp: &mut Interpreter,
// ) -> PartialVMResult<()> {
//     let frame_index = interp.call_stack.0.len();
//     let module_id = function_desc.module_id();
//     let function_index = function_desc.index();
//     let stack_pointer = interp.operand_stack.value.len();
//
//     let caller_frame = interp.call_stack.0.last();
//     match instr {
//         Bytecode::Pop => {
//             let val = interp.operand_stack.last_n(1)?.last().unwrap();
//             Footprint::Pop {
//                 poped_value: val.into(),
//             }
//         }
//         Bytecode::Ret => Footprint::Ret,
//         Bytecode::BrTrue(_) => {
//             let val = interp
//                 .operand_stack
//                 .last_n(1)?
//                 .last()
//                 .unwrap()
//                 .as_value_ref::<bool>()?
//                 .clone();
//             Footprint::BrTrue { cond_val: val }
//         }
//         Bytecode::BrFalse(_) => {
//             let val = interp
//                 .operand_stack
//                 .last_n(1)?
//                 .last()
//                 .unwrap()
//                 .as_value_ref::<bool>()?
//                 .clone();
//             Footprint::BrTrue { cond_val: val }
//         }
//         Bytecode::Branch(_) => Footprint::Branch,
//         Bytecode::LdU8(_) | Bytecode::LdU64(_) | Bytecode::LdU128(_) => Footprint::Ld,
//         Bytecode::CastU8 => {}
//         Bytecode::CastU64 => {}
//         Bytecode::CastU128 => {}
//         Bytecode::LdConst(_) => {}
//         Bytecode::LdTrue => {}
//         Bytecode::LdFalse => {}
//         Bytecode::CopyLoc(idx) => {
//             let local = locals.copy_loc(*idx as usize)?;
//             Footprint::CopyLoc {
//                 local: (&local).into(),
//             }
//         }
//         Bytecode::MoveLoc(idx) => {
//             let local = locals.copy_loc(*idx as usize)?;
//             Footprint::MoveLoc {
//                 local: (&local).into(),
//             }
//         }
//         Bytecode::StLoc(idx) => {
//             let local = locals.copy_loc(*idx as usize)?;
//             let new_value = interp.operand_stack.last_n(1)?.last().unwrap().into();
//             Footprint::StLoc {
//                 old_local: (&local).into(),
//                 new_value,
//             }
//             // TODO: value stored to loc only have 1 reference on it
//             // so we can hook here to index every sub items by it rc-ptr.
//         }
//         Bytecode::Call(fh_idx) => {
//             let func = resolver
//                 .function_from_handle(*fh_idx)
//                 .map_err(|e| interp.set_location(e))?;
//             Footprint::Call {
//                 args: interp
//                     .operand_stack
//                     .last_n(func.param_count())?
//                     .map(|t| t.into())
//                     .collect::<Vec<_>>(),
//             }
//         }
//         Bytecode::CallGeneric(_) => {
//             unimplemented!()
//         }
//         Bytecode::Pack(_) => {}
//         Bytecode::PackGeneric(_) => {}
//         Bytecode::Unpack(_) => {}
//         Bytecode::UnpackGeneric(_) => {}
//         Bytecode::ReadRef => {}
//         Bytecode::WriteRef => {}
//         Bytecode::FreezeRef => {}
//         Bytecode::MutBorrowLoc(_) => Footprint::MutBorrowLoc,
//         Bytecode::ImmBorrowLoc(_) => Footprint::ImmBorrowLoc,
//         Bytecode::MutBorrowField(fh_idx) => {
//             let reference = interp.operand_stack.pop_as::<StructRef>()?;
//             reference.raw_address();
//             let offset = resolver.field_offset(*fh_idx);
//             let field_ref = reference.borrow_field(offset)?;
//         }
//         Bytecode::MutBorrowFieldGeneric(_) => {}
//         Bytecode::ImmBorrowField(_) => {}
//         Bytecode::ImmBorrowFieldGeneric(_) => {}
//         Bytecode::MutBorrowGlobal(_) => {}
//         Bytecode::MutBorrowGlobalGeneric(_) => {}
//         Bytecode::ImmBorrowGlobal(_) => {}
//         Bytecode::ImmBorrowGlobalGeneric(_) => {}
//         Bytecode::Add => {}
//         Bytecode::Sub => {}
//         Bytecode::Mul => {}
//         Bytecode::Mod => {}
//         Bytecode::Div => {}
//         Bytecode::BitOr => {}
//         Bytecode::BitAnd => {}
//         Bytecode::Xor => {}
//         Bytecode::Or => {}
//         Bytecode::And => {}
//         Bytecode::Not => {}
//         Bytecode::Eq => {}
//         Bytecode::Neq => {}
//         Bytecode::Lt => {}
//         Bytecode::Gt => {}
//         Bytecode::Le => {}
//         Bytecode::Ge => {}
//         Bytecode::Abort => {}
//         Bytecode::Nop => {}
//         Bytecode::Exists(_) => {}
//         Bytecode::ExistsGeneric(_) => {}
//         Bytecode::MoveFrom(_) => {}
//         Bytecode::MoveFromGeneric(_) => {}
//         Bytecode::MoveTo(_) => {}
//         Bytecode::MoveToGeneric(_) => {}
//         Bytecode::Shl => {}
//         Bytecode::Shr => {}
//         Bytecode::VecPack(_, _) => {}
//         Bytecode::VecLen(_) => {}
//         Bytecode::VecImmBorrow(_) => {}
//         Bytecode::VecMutBorrow(_) => {}
//         Bytecode::VecPushBack(_) => {}
//         Bytecode::VecPopBack(_) => {}
//         Bytecode::VecUnpack(_, _) => {}
//         Bytecode::VecSwap(_) => {}
//         Bytecode::LdU16(_) => {}
//         Bytecode::LdU32(_) => {}
//         Bytecode::LdU256(_) => {}
//         Bytecode::CastU16 => {}
//         Bytecode::CastU32 => {}
//         Bytecode::CastU256 => {}
//     };
//
//     Ok(())
// }
