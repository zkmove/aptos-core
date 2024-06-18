use std::collections::BTreeMap;

use serde::{Deserialize, Serialize};

use move_core_types::{account_address::AccountAddress, u256, u256::U256};
use move_vm_types::{
    delayed_values::delayed_field_id::DelayedFieldID,
    values::Value,
    views::{ValueView, ValueVisitor},
};
use move_vm_types::values::IntegerValue;

#[derive(Clone, Debug, Serialize, Deserialize)]
pub enum SimpleValue {
    U8(u8),
    U16(u16),
    U32(u32),
    U64(u64),
    U128(u128),
    U256(U256),
    Bool(bool),
    Address(AccountAddress),
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub enum Integer {
    U8(u8),
    U16(u16),
    U32(u32),
    U64(u64),
    U128(u128),
    U256(U256),
}


impl From<IntegerValue> for Integer {
    fn from(value: IntegerValue) -> Self {
        match value {
            IntegerValue::U8(v) => { Self::U8(v) }
            IntegerValue::U16(v) => { Self::U16(v) }
            IntegerValue::U32(v) => Self::U32(v),
            IntegerValue::U64(v) => Self::U64(v),
            IntegerValue::U128(v) => Self::U128(v),
            IntegerValue::U256(v) => Self::U256(v)
        }
    }
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct Reference {
    pub frame_index: usize,
    pub local_index: usize,
    pub sub_index: Vec<usize>,
}

impl Reference {
    pub fn new(frame_index: usize, local_index: usize, sub_index: Vec<usize>) -> Self {
        Reference {
            frame_index,
            local_index,
            sub_index,
        }
    }
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct ValueItem {
    sub_index: Vec<usize>,
    header: bool,
    value: SimpleValue,
}

#[derive(Copy, Clone)]
struct FrameState {
    depth: usize,
    len: usize,
    counter: usize,
}

#[derive(Clone, Default)]
pub struct TracedValue {
    visit_stack: Vec<FrameState>,
    items: Vec<ValueItem>,
    container_sub_indexes: BTreeMap<usize, Vec<usize>>,
}

pub type ValueItems = Vec<ValueItem>;

impl TracedValue {
    pub fn items(&self) -> Vec<ValueItem> {
        assert!(self.visit_stack.is_empty());
        self.items.clone()
    }

    pub fn container_sub_indexes(&self) -> BTreeMap<usize, Vec<usize>> {
        assert!(self.visit_stack.is_empty());
        self.container_sub_indexes.clone()
    }
}

impl From<&Value> for TracedValue {
    fn from(value: &Value) -> Self {
        let mut this = Self::default();
        value.visit(&mut this);
        this
    }
}

impl TracedValue {
    pub fn current_sub_index(&self) -> Vec<usize> {
        self.visit_stack.iter().map(|s| s.counter).collect()
    }
}

impl TracedValue {
    fn visit_simple(&mut self, depth: usize, value: SimpleValue) {
        let sub_index = match self.visit_stack.last_mut() {
            Some(frame) => {
                frame.counter += 1;
                assert_eq!(frame.depth + 1, depth);
                self.current_sub_index()
            },
            None => {
                assert_eq!(depth, 0);
                vec![0]
            },
        };
        self.items.push(ValueItem {
            sub_index,
            header: false,
            value,
        });

        // trace-up to the top un-finished frame
        while self
            .visit_stack
            .last()
            .filter(|s| s.counter == s.len)
            .is_some()
        {
            self.visit_stack.pop();
        }
    }
}

impl ValueVisitor for TracedValue {
    fn visit_delayed(&mut self, _depth: usize, _id: DelayedFieldID) {
        unreachable!()
    }

    fn visit_u8(&mut self, depth: usize, val: u8) {
        self.visit_simple(depth, SimpleValue::U8(val))
    }

    fn visit_u16(&mut self, depth: usize, val: u16) {
        self.visit_simple(depth, SimpleValue::U16(val))
    }

    fn visit_u32(&mut self, depth: usize, val: u32) {
        self.visit_simple(depth, SimpleValue::U32(val))
    }

    fn visit_u64(&mut self, depth: usize, val: u64) {
        self.visit_simple(depth, SimpleValue::U64(val))
    }

    fn visit_u128(&mut self, depth: usize, val: u128) {
        self.visit_simple(depth, SimpleValue::U128(val))
    }

    fn visit_u256(&mut self, depth: usize, val: u256::U256) {
        self.visit_simple(depth, SimpleValue::U256(val))
    }

    fn visit_bool(&mut self, depth: usize, val: bool) {
        self.visit_simple(depth, SimpleValue::Bool(val))
    }

    fn visit_address(&mut self, depth: usize, val: AccountAddress) {
        self.visit_simple(depth, SimpleValue::Address(val))
    }

    fn visit_container(&mut self, raw_address: usize, depth: usize) {
        match self.visit_stack.last_mut() {
            Some(last_frame) => {
                last_frame.counter += 1;
                assert_eq!(last_frame.depth + 1, depth);
            },
            None => {
                assert_eq!(depth, 0);
            },
        }
        let mut sub_index = self.current_sub_index();
        sub_index.push(0);
        self.container_sub_indexes.insert(raw_address, sub_index);
    }

    fn visit_struct(&mut self, depth: usize, len: usize) -> bool {
        match self.visit_stack.last_mut() {
            Some(last_frame) => {
                last_frame.counter += 1;
                assert_eq!(last_frame.depth + 1, depth);
            },
            None => {
                assert_eq!(depth, 0);
            },
        }
        let new_frame = FrameState {
            depth,
            len,
            counter: 0,
        };
        self.visit_stack.push(new_frame);
        self.items.push(ValueItem {
            header: true,
            sub_index: self.current_sub_index(),
            value: SimpleValue::U64(len as u64),
        });
        true
    }

    fn visit_vec(&mut self, depth: usize, len: usize) -> bool {
        let new_frame = FrameState {
            depth,
            len,
            counter: 0,
        };
        self.visit_stack.push(new_frame);
        self.items.push(ValueItem {
            header: true,
            sub_index: self.current_sub_index(),
            value: SimpleValue::U64(len as u64),
        });
        true
    }

    fn visit_ref(&mut self, _depth: usize, _is_global: bool) -> bool {
        panic!("ref cannot be a field of container")
    }
}

#[derive(Copy, Clone, Default)]
pub(crate) struct ReferenceValueVisitor {
    pub(crate) reference_pointer: usize,
    pub(crate) indexed: Option<usize>,
}

impl ValueVisitor for ReferenceValueVisitor {
    fn visit_delayed(&mut self, _depth: usize, _id: DelayedFieldID) {}

    fn visit_u8(&mut self, _depth: usize, _val: u8) {}

    fn visit_u16(&mut self, _depth: usize, _val: u16) {}

    fn visit_u32(&mut self, _depth: usize, _val: u32) {}

    fn visit_u64(&mut self, _depth: usize, _val: u64) {}

    fn visit_u128(&mut self, _depth: usize, _val: u128) {}

    fn visit_u256(&mut self, _depth: usize, _val: U256) {}

    fn visit_bool(&mut self, _depth: usize, _val: bool) {}

    fn visit_address(&mut self, _depth: usize, _val: AccountAddress) {}

    fn visit_struct(&mut self, _depth: usize, _len: usize) -> bool {
        false
    }

    fn visit_vec(&mut self, _depth: usize, _len: usize) -> bool {
        false
    }

    fn visit_ref(&mut self, _depth: usize, _is_global: bool) -> bool {
        true
    }

    fn visit_container(&mut self, raw_address: usize, depth: usize) {
        if depth == 1 {
            self.reference_pointer = raw_address;
        } else {
            unreachable!()
        }
    }

    fn visit_indexed(&mut self, raw_address: usize, depth: usize, idx: usize) {
        if depth == 0 {
            self.reference_pointer = raw_address;
            self.indexed = Some(idx);
        } else {
            unreachable!()
        }
    }
}
