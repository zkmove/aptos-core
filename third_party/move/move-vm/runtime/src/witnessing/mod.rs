use move_binary_format::errors::PartialVMError;
use move_binary_format::file_format::Bytecode;
use move_core_types::language_storage::ModuleId;
use move_core_types::vm_status::StatusCode;
use move_vm_types::values::IntegerValue;

use crate::witnessing::traced_value::{Reference, ValueItems};

pub mod traced_value;

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
    LdConst,
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
    CallGeneric {
        args: Vec<ValueItems>,
    },
    Pack {
        args: Vec<ValueItems>,
    },
    PackGeneric {
        args: Vec<ValueItems>,
    },
    Unpack {
        arg: ValueItems,
    },
    UnpackGeneric {
        arg: ValueItems,
    },
    ReadRef {
        reference: Reference,
        value: ValueItems,
    },
    WriteRef {
        reference: Reference,
        old_value: ValueItems,
        new_value: ValueItems,
    },
    FreezeRef,
    BinaryOp {
        ty: BinaryIntegerOperationType,
        lhs: IntegerValue,
        rhs: IntegerValue,
    },
    Or {
        lhs: bool,
        rhs: bool,
    },
    And {
        lhs: bool,
        rhs: bool,
    },
    Not {
        value: bool,
    },
    Shl {
        rhs: u8,
        lhs: IntegerValue,
    },
    Shr {
        rhs: u8,
        lhs: IntegerValue,
    },
    Eq {
        lhs: ValueItems,
        rhs: ValueItems,
    },
    Neq {
        lhs: ValueItems,
        rhs: ValueItems,
    },
    Abort {
        error_code: u64,
    },
    Nop,
    VecPack {
        args: Vec<ValueItems>,
    },
    VecUnpack {
        arg: ValueItems,
    },
    VecLen {
        vec_ref: Reference,
        len: u64,
    },
    VecBorrow {
        imm: bool,
        idx: u64,
        vec_ref: Reference,
        elem: ValueItems,
    },
    VecPushBack {
        vec_len: u64,
        vec_ref: Reference,
        elem: ValueItems,
    },
    VecPopBack {
        vec_len: u64,
        vec_ref: Reference,
        elem: ValueItems,
    },

    VecSwap {
        vec_ref: Reference,
        vec_len: u64,
        idx1: u64,
        idx2: u64,
        idx1_elem: ValueItems,
        idx2_elem: ValueItems,
    },
    BorrowLoc {
        imm: bool,
    },
    ImmBorrowLoc,
    BorrowField {
        imm: bool,
        reference: Reference,
        field_offset: usize,
    },
    BorrowFieldGeneric {
        imm: bool,
        reference: Reference,
        field_offset: usize,
    },
    CastU8 {
        origin: IntegerValue,
    },
    CastU16 {
        origin: IntegerValue,
    },
    CastU32 {
        origin: IntegerValue,
    },
    CastU64 {
        origin: IntegerValue,
    },
    CastU128 {
        origin: IntegerValue,
    },
    CastU256 {
        origin: IntegerValue,
    },
}

#[derive(Copy, Clone, Debug)]
pub enum BinaryIntegerOperationType {
    Add,
    Sub,
    Mul,
    Mod,
    Div,
    BitOr,
    BitAnd,
    Xor,
    Lt,
    Gt,
    Le,
    Ge,
}

impl TryFrom<Bytecode> for BinaryIntegerOperationType {
    type Error = PartialVMError;

    fn try_from(value: Bytecode) -> Result<Self, Self::Error> {
        Ok(match value {
            Bytecode::Add => BinaryIntegerOperationType::Add,
            Bytecode::Sub => BinaryIntegerOperationType::Sub,
            Bytecode::Mul => BinaryIntegerOperationType::Mul,
            Bytecode::Mod => BinaryIntegerOperationType::Mod,
            Bytecode::Div => BinaryIntegerOperationType::Div,
            Bytecode::BitOr => BinaryIntegerOperationType::BitOr,
            Bytecode::BitAnd => BinaryIntegerOperationType::BitAnd,
            Bytecode::Xor => BinaryIntegerOperationType::Xor,
            Bytecode::Lt => BinaryIntegerOperationType::Lt,
            Bytecode::Gt => BinaryIntegerOperationType::Gt,
            Bytecode::Le => BinaryIntegerOperationType::Le,
            Bytecode::Ge => BinaryIntegerOperationType::Ge,
            _ => {
                return Err(PartialVMError::new(StatusCode::INTERNAL_TYPE_ERROR)
                    .with_message(format!("{:?} is not a binary operation", value)));
            }
        })
    }
}

#[derive(Clone, Debug)]
pub struct Footprint {
    pub module_id: Option<ModuleId>,
    pub function_id: usize,
    pub pc: u16,
    pub frame_index: usize,
    pub stack_pointer: usize,
    pub op: Bytecode,
    pub data: Operation,
}

