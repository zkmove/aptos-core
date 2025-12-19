// Copyright (c) The Diem Core Contributors
// Copyright (c) The Move Contributors
// Copyright (c) The zkMove Contributors
// SPDX-License-Identifier: Apache-2.0

use aptos_gas_schedule::gas_params::natives::move_stdlib::ZKHASH_POSEIDON_BASE;
use aptos_native_interface::{safely_pop_arg, RawSafeNative, SafeNativeBuilder, SafeNativeContext, SafeNativeResult};
use halo2curves::bn256::Fr;
use halo2curves::ff::PrimeField;
use move_core_types::int256::U256;
use move_vm_runtime::native_functions::NativeFunction;
use move_vm_types::{
    loaded_data::runtime_types::Type,
    values::Value,
};
use smallvec::{smallvec, SmallVec};
use std::collections::VecDeque;

/***************************************************************************************************
 * native poseidon_hash
 **************************************************************************************************/
const DOMAIN_SPEC: u64 = 1; // Domain spec for Poseidon hash
fn native_poseidon_hash(
    context: &mut SafeNativeContext,
    _ty_args: Vec<Type>,
    mut args: VecDeque<Value>,
) -> SafeNativeResult<SmallVec<[Value; 1]>> {
    debug_assert!(_ty_args.is_empty());
    debug_assert!(args.len() == 2);

    context.charge(ZKHASH_POSEIDON_BASE)?;

    let arg2 = safely_pop_arg!(args, u128);
    let arg1 = safely_pop_arg!(args, u128);

    let hash_result = poseidon_base::Hashable::hash_with_domain([Fr::from_u128(arg1), Fr::from_u128(arg2)], Fr::from(DOMAIN_SPEC));
    let hash_val = U256::from_le_bytes(hash_result.to_repr().as_ref().try_into().unwrap());

    Ok(smallvec![Value::u256(hash_val)])
}

/***************************************************************************************************
    * module
    **************************************************************************************************/
pub fn make_all(
    builder: &SafeNativeBuilder,
) -> impl Iterator<Item = (String, NativeFunction)> + '_ {
    let natives = [
        ("poseidon_hash", crate::natives::zkhash::native_poseidon_hash as RawSafeNative),
    ];

    builder.make_named_natives(natives)
}
