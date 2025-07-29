// Copyright (c) The Diem Core Contributors
// Copyright (c) The Move Contributors
// SPDX-License-Identifier: Apache-2.0

use crate::natives::helpers::make_module_natives;
use halo2curves::bn256::Fr;
use halo2curves::ff::PrimeField;
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
 * native poseidon_hash
 **************************************************************************************************/
#[derive(Debug, Clone)]
pub struct PoseidonHashGasParameters {
    pub base: InternalGas,
}
const DOMAIN_SPEC: u64 = 1; // Domain spec for Poseidon hash
fn native_poseidon_hash(
    gas_params: &PoseidonHashGasParameters,
    _context: &mut NativeContext,
    _ty_args: Vec<Type>,
    mut args: VecDeque<Value>,
) -> PartialVMResult<NativeResult> {
    debug_assert!(_ty_args.is_empty());
    debug_assert!(args.len() == 2);

    let cost = gas_params.base;

    let arg2 = pop_arg!(args, u128);
    let arg1 = pop_arg!(args, u128);

    let hash_result = poseidon_base::Hashable::hash_with_domain([Fr::from_u128(arg1), Fr::from_u128(arg2)], Fr::from(DOMAIN_SPEC));
    let hash_val = U256::from_le_bytes(&hash_result.to_repr());

    Ok(NativeResult::ok(cost, smallvec![Value::u256(hash_val)]))
}

pub fn make_native_poseidon_hash(gas_params: PoseidonHashGasParameters) -> NativeFunction {
    Arc::new(
        move |context, ty_args, args| -> PartialVMResult<NativeResult> {
            native_poseidon_hash(&gas_params, context, ty_args, args)
        },
    )
}

/***************************************************************************************************
 * module
 **************************************************************************************************/
#[derive(Debug, Clone)]
pub struct GasParameters {
    pub poseidon_hash: PoseidonHashGasParameters,
}

pub fn make_all(gas_params: GasParameters) -> impl Iterator<Item=(String, NativeFunction)> {
    let natives = [
        ("poseidon_hash", make_native_poseidon_hash(gas_params.poseidon_hash)),
    ];

    make_module_natives(natives)
}
