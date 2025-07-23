// Copyright (c) The Diem Core Contributors
// Copyright (c) The Move Contributors
// SPDX-License-Identifier: Apache-2.0

use crate::natives::helpers::make_module_natives;
use move_binary_format::errors::PartialVMResult;
use move_core_types::gas_algebra::InternalGas;
use move_core_types::u256::U256;
use move_vm_runtime::native_functions::{NativeContext, NativeFunction};
use move_vm_types::{
    loaded_data::runtime_types::Type,
    natives::function::NativeResult,
    pop_arg,
    values::Value,
};
use smallvec::smallvec;
use std::collections::VecDeque;
use std::sync::Arc;
/***************************************************************************************************
 * native fake_hash
 **************************************************************************************************/
#[derive(Debug, Clone)]
pub struct FakeHashGasParameters {
    pub base: InternalGas,
}

fn native_fake_hash(
    gas_params: &FakeHashGasParameters,
    _context: &mut NativeContext,
    _ty_args: Vec<Type>,
    mut args: VecDeque<Value>,
) -> PartialVMResult<NativeResult> {
    debug_assert!(_ty_args.is_empty());
    debug_assert!(args.len() == 2);

    let cost = gas_params.base;

    let arg2 = pop_arg!(args, u128);
    let arg1 = pop_arg!(args, u128);

    let mut hash_vec = [0u8; 32];
    hash_vec[0..16].copy_from_slice(&arg1.to_le_bytes());
    hash_vec[16..32].copy_from_slice(&arg2.to_le_bytes());
    let hash_val = U256::from_le_bytes(&hash_vec);
    Ok(NativeResult::ok(cost, smallvec![Value::u256(hash_val)]))
}

pub fn make_native_fake_hash(gas_params: FakeHashGasParameters) -> NativeFunction {
    Arc::new(
        move |context, ty_args, args| -> PartialVMResult<NativeResult> {
            native_fake_hash(&gas_params, context, ty_args, args)
        },
    )
}

/***************************************************************************************************
 * module
 **************************************************************************************************/
#[derive(Debug, Clone)]
pub struct GasParameters {
    pub fake_hash: FakeHashGasParameters,
}

pub fn make_all(gas_params: GasParameters) -> impl Iterator<Item=(String, NativeFunction)> {
    let natives = [
        ("fake_hash", make_native_fake_hash(gas_params.fake_hash)),
    ];

    make_module_natives(natives)
}
