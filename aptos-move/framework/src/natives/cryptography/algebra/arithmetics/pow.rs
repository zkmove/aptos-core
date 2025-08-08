// Copyright © Aptos Foundation
// SPDX-License-Identifier: Apache-2.0

use crate::{
    abort_unless_arithmetics_enabled_for_structure, abort_unless_feature_flag_enabled,
    natives::cryptography::algebra::{
        abort_invariant_violated, feature_flag_from_structure, AlgebraContext, Structure,
        E_TOO_MUCH_MEMORY_USED, MEMORY_LIMIT_IN_BYTES, MOVE_ABORT_CODE_NOT_IMPLEMENTED,
    },
    safe_borrow_element, store_element, structure_from_ty_arg,
};
use aptos_gas_schedule::gas_params::natives::aptos_framework::*;
use aptos_native_interface::{safely_pop_arg, SafeNativeContext, SafeNativeError, SafeNativeResult};
use ark_ff::Field;
use move_vm_types::{
    loaded_data::runtime_types::Type,
    values::{Value, VectorRef},
};
use smallvec::{smallvec, SmallVec};
use std::{collections::VecDeque, rc::Rc};

macro_rules! ark_pow_internal {
    ($context:expr, $args:ident, $group_typ:ty, $op:ident, $gas:expr) => {{
        let exp_ref = safely_pop_arg!($args, VectorRef);
        let vec_u64_ref = exp_ref.as_vec_u64_ref();
        let exp_limbs = vec_u64_ref.as_slice();

        let handle = safely_pop_arg!($args, u64) as usize;
        safe_borrow_element!($context, handle, $group_typ, element_ptr, element);

        $context.charge($gas)?;
        let new_element = element.$op(exp_limbs);
        let new_handle = store_element!($context, new_element)?;
        Ok(smallvec![Value::u64(new_handle as u64)])
    }};
}

pub fn pow_internal(
    context: &mut SafeNativeContext,
    ty_args: Vec<Type>,
    mut args: VecDeque<Value>,
) -> SafeNativeResult<SmallVec<[Value; 1]>> {
    assert_eq!(1, ty_args.len());
    let structure_opt = structure_from_ty_arg!(context, &ty_args[0]);
    abort_unless_arithmetics_enabled_for_structure!(context, structure_opt);

    match structure_opt {
        Some(Structure::BLS12381Fr) => {
            ark_pow_internal!(
                context,
                args,
                ark_bls12_381::Fr,
                pow,
                ALGEBRA_ARK_BLS12_381_FR_POW
            )
        },
        Some(Structure::BLS12381Fq12) => {
            ark_pow_internal!(
                context,
                args,
                ark_bls12_381::Fq12,
                pow,
                ALGEBRA_ARK_BLS12_381_FQ12_POW
            )
        }
        Some(Structure::BN254Fr) => {
            ark_pow_internal!(
                context,
                args,
                ark_bn254::Fr,
                pow,
                ALGEBRA_ARK_BN254_FR_POW
            )
        }
        Some(Structure::BN254Fq) => {
            ark_pow_internal!(
                context,
                args,
                ark_bn254::Fq,
                pow,
                ALGEBRA_ARK_BN254_FQ_POW
            )
        }
        Some(Structure::BN254Fq12) => {
            ark_pow_internal!(
                context,
                args,
                ark_bn254::Fq12,
                pow,
                ALGEBRA_ARK_BN254_FQ12_POW
            )
        }
        _ => Err(SafeNativeError::Abort {
            abort_code: MOVE_ABORT_CODE_NOT_IMPLEMENTED,
        }),
    }
}