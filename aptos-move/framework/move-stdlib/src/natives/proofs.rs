// Copyright (c) The zkMove Contributors
// SPDX-License-Identifier: Apache-2.0

use aptos_gas_schedule::gas_params::natives::move_stdlib::{PROOFS_HALO2_BASE, PROOFS_HALO2_PER_BYTE};
use aptos_native_interface::{
    safely_pop_arg, RawSafeNative, SafeNativeBuilder, SafeNativeContext,
    SafeNativeResult,
};
use move_core_types::gas_algebra::NumBytes;
use move_vm_runtime::native_functions::NativeFunction;
use move_vm_types::{
    loaded_data::runtime_types::Type,
    values::Value,
};
use smallvec::{smallvec, SmallVec};
use std::collections::VecDeque;

use halo2_verifier::deserialize_circuit_and_verify;

/***************************************************************************************************
 * native verify_halo2_proof
 **************************************************************************************************/
fn native_verify_halo2_proof(
    context: &mut SafeNativeContext,
    ty_args: Vec<Type>,
    mut args: VecDeque<Value>,
) -> SafeNativeResult<SmallVec<[Value; 1]>> {
    debug_assert!(ty_args.is_empty());
    debug_assert_eq!(args.len(), 8);

    context.charge(PROOFS_HALO2_BASE)?;

    let mut total_bytes: u64 = 0;

    let k = safely_pop_arg!(args, u32);
    let k_present = safely_pop_arg!(args, bool);
    let k_opt = if k_present {
        Some(k)
    } else {
        None
    };
    let kzg = safely_pop_arg!(args, u8);

    let proof = safely_pop_arg!(args, Vec<u8>);
    total_bytes += proof.len() as u64;

    let public_inputs = safely_pop_arg!(args, Vec<u8>);
    let circuit_info = safely_pop_arg!(args, Vec<u8>);
    total_bytes += circuit_info.len() as u64 + public_inputs.len() as u64;

    let vk_bytes = safely_pop_arg!(args, Vec<u8>);
    total_bytes += vk_bytes.len() as u64;
    let params = safely_pop_arg!(args, Vec<u8>);
    total_bytes += params.len() as u64;

    let gas = PROOFS_HALO2_PER_BYTE
        * NumBytes::new(total_bytes.max(1024) as u64); // at least 1KB
    context.charge(gas)?;

    let result = deserialize_circuit_and_verify(
        &params,
        &vk_bytes,
        &circuit_info,
        &public_inputs,
        &proof,
        kzg,
        k_opt,
    );
    let success = result.is_ok();
    Ok(smallvec![Value::bool(success)])
}

/***************************************************************************************************
 * module registration
 **************************************************************************************************/
pub fn make_all(
    builder: &SafeNativeBuilder,
) -> impl Iterator<Item = (String, NativeFunction)> + '_ {
    let natives = [(
        "verify_halo2_proof",
        native_verify_halo2_proof as RawSafeNative,
    )];

    builder.make_named_natives(natives)
}
