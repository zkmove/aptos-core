use move_core_types::{account_address::AccountAddress, u256, u256::U256};
use move_vm_types::{
    delayed_values::delayed_field_id::DelayedFieldID,
    values::{IntegerValue, Value},
    views::{ValueView, ValueVisitor},
};
use serde::{Deserialize, Serialize};
use std::collections::BTreeMap;

#[derive(Clone, Debug, Serialize, Deserialize, PartialEq, Eq)]
pub enum SimpleValue {
    U8(u8),
    U16(u16),
    U32(u32),
    U64(u64),
    U128(u128),
    U256(U256),
    Bool(bool),
    Address(AccountAddress),
    Reference(Reference),
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
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
            IntegerValue::U8(v) => Self::U8(v),
            IntegerValue::U16(v) => Self::U16(v),
            IntegerValue::U32(v) => Self::U32(v),
            IntegerValue::U64(v) => Self::U64(v),
            IntegerValue::U128(v) => Self::U128(v),
            IntegerValue::U256(v) => Self::U256(v),
        }
    }
}

impl From<Integer> for IntegerValue {
    fn from(value: Integer) -> Self {
        match value {
            Integer::U8(v) => Self::U8(v),
            Integer::U16(v) => Self::U16(v),
            Integer::U32(v) => Self::U32(v),
            Integer::U64(v) => Self::U64(v),
            Integer::U128(v) => Self::U128(v),
            Integer::U256(v) => Self::U256(v),
        }
    }
}

impl From<Integer> for SimpleValue {
    fn from(value: Integer) -> Self {
        match value {
            Integer::U8(v) => Self::U8(v),
            Integer::U16(v) => Self::U16(v),
            Integer::U32(v) => Self::U32(v),
            Integer::U64(v) => Self::U64(v),
            Integer::U128(v) => Self::U128(v),
            Integer::U256(v) => Self::U256(v),
        }
    }
}

impl TryFrom<SimpleValue> for Integer {
    type Error = anyhow::Error;

    fn try_from(value: SimpleValue) -> anyhow::Result<Integer> {
        match value {
            SimpleValue::U8(v) => Ok(Integer::U8(v)),
            SimpleValue::U16(v) => Ok(Integer::U16(v)),
            SimpleValue::U32(v) => Ok(Integer::U32(v)),
            SimpleValue::U64(v) => Ok(Integer::U64(v)),
            SimpleValue::U128(v) => Ok(Integer::U128(v)),
            SimpleValue::U256(v) => Ok(Integer::U256(v)),
            _ => Err(anyhow::anyhow!(
                "Invalid SimpleValue type for converting into Integer"
            )),
        }
    }
}

/// Return lower and higher 128-bits of of an Integer as u128 pair
impl From<Integer> for (u128 /*lo*/, u128 /*hi*/) {
    fn from(value: Integer) -> (u128, u128) {
        match value {
            Integer::U8(v) => (v as u128, 0u128),
            Integer::U16(v) => (v as u128, 0u128),
            Integer::U32(v) => (v as u128, 0u128),
            Integer::U64(v) => (v as u128, 0u128),
            Integer::U128(v) => (v as u128, 0u128),
            Integer::U256(v) => {
                let bytes = v.to_le_bytes();
                let lo = u128::from_le_bytes(bytes[..16].try_into().unwrap());
                let hi = u128::from_le_bytes(bytes[16..].try_into().unwrap());
                (lo, hi)
            },
        }
    }
}

const MAX_SUB_INDEX_DEPTH: usize = 8;

#[derive(Clone, Debug, Ord, PartialOrd, PartialEq, Eq, Serialize, Deserialize)]
pub struct SubIndex([u16; MAX_SUB_INDEX_DEPTH]);

impl SubIndex {
    pub fn depth(&self) -> usize {
        self.0
            .iter()
            .rposition(|&x| x != 0)
            .map_or(0, |pos| pos + 1)
    }

    /// A depth-n sub_index must have n parents. Return all parents in a vector, in a order
    /// starting with direct relatives. For example,
    /// [1,2,3,0]'s parents is [[1,2,0,0],[1,0,0,0],[0,0,0,0]]
    pub fn parents(&self) -> Vec<Self> {
        let depth = self.depth();
        let mut parent = self.0;
        let mut parents = Vec::with_capacity(depth);

        for i in (0..depth).rev() {
            parent[i] = 0;
            parents.push(SubIndex(parent));
        }

        parents
    }

    /// Trim tailing zeros of sub_index and concat with other sub_index. For example,
    /// let sub_index = [3,2,0,0,0,0,0,0];
    /// let other = [4,1,0,0,0,0,0,0];
    /// sub_index.concat(other) = [3,2,4,1,0,0,0,0];
    pub fn concat(&self, other: &SubIndex) -> Self {
        let mut this = self.0.to_vec();
        let other = other.0.to_vec();

        // Remove trailing zeros
        while this.last() == Some(&0) {
            this.pop();
        }

        this.extend(other);

        let mut result = [0; MAX_SUB_INDEX_DEPTH];
        for (i, &val) in this.iter().enumerate().take(MAX_SUB_INDEX_DEPTH) {
            result[i] = val;
        }

        SubIndex(result)
    }

    pub fn push(&mut self, element: u16) {
        // Find the first zero element to replace
        if let Some(position) = self.0.iter().position(|&x| x == 0) {
            self.0[position] = element;
        } else {
            panic!("SubIndex is full");
        }
    }

    pub fn insert(&mut self, index: usize, element: u16) {
        assert!(index < MAX_SUB_INDEX_DEPTH, "Index out of bounds");

        // Shift elements to the right, starting from the last element to the index
        for i in (index..MAX_SUB_INDEX_DEPTH - 1).rev() {
            self.0[i + 1] = self.0[i];
        }

        self.0[index] = element;
    }

    pub fn remove(&mut self, index: usize) -> u16 {
        assert!(index < MAX_SUB_INDEX_DEPTH, "Index out of bounds");
        let removed_element = self.0[index];

        // Shift elements to the left
        for i in index..MAX_SUB_INDEX_DEPTH - 1 {
            self.0[i] = self.0[i + 1];
        }

        self.0[MAX_SUB_INDEX_DEPTH - 1] = 0;
        removed_element
    }

    pub fn to_vec(&self) -> Vec<u16> {
        self.0.to_vec()
    }

    pub fn to_trimmed_vec(&self) -> Vec<u16> {
        let mut vec = self.0.to_vec();

        // Remove trailing zeros but keep a single zero if it's the only element
        while vec.len() > 1 && vec.last() == Some(&0) {
            vec.pop();
        }

        vec
    }
}

impl From<Vec<usize>> for SubIndex {
    fn from(value: Vec<usize>) -> Self {
        assert!(
            value.len() <= MAX_SUB_INDEX_DEPTH,
            "Input vector length exceeds MAX_SUB_INDEX_DEPTH"
        );
        let mut result = [0; MAX_SUB_INDEX_DEPTH];

        for (i, &val) in value.iter().enumerate() {
            assert!(val <= u16::MAX as usize, "Value {} exceeds u16::MAX", val);
            result[i] = val as u16;
        }

        SubIndex(result)
    }
}

/// Convert SubIndex into u128 in little endian order
impl From<SubIndex> for u128 {
    fn from(sub_index: SubIndex) -> u128 {
        let mut result = 0u128;
        for (i, &value) in sub_index.0.iter().enumerate() {
            result |= (value as u128) << (i * 16);
        }
        result
    }
}

impl From<u128> for SubIndex {
    fn from(value: u128) -> Self {
        let mut result = [0u16; MAX_SUB_INDEX_DEPTH];

        for i in 0..MAX_SUB_INDEX_DEPTH {
            result[i] = ((value >> (i * 16)) & 0xFFFF) as u16;
        }

        SubIndex(result)
    }
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct Reference {
    pub frame_index: usize,
    pub local_index: usize,
    pub sub_index: SubIndex,
}

impl Reference {
    pub fn new(frame_index: usize, local_index: usize, sub_index: SubIndex) -> Self {
        Reference {
            frame_index,
            local_index,
            sub_index,
        }
    }
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct ValueItem {
    pub sub_index: SubIndex,
    pub header: bool,
    pub value: SimpleValue,
}

#[derive(Copy, Clone, PartialEq, Eq)]
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
            sub_index: sub_index.into(),
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
            sub_index: self.current_sub_index().into(),
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
            sub_index: self.current_sub_index().into(),
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
