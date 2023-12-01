// Copyright © Aptos Foundation

// Copyright (c) Aptos
// SPDX-License-Identifier: Apache-2.0

#[macro_use]
extern crate criterion;

use aptos_crypto::test_utils::random_bytes;
use ark_bn254::{Fq12, Fr, G1Affine, G1Projective, G2Affine, G2Projective};
use ark_ec::{
    hashing::HashToCurve, pairing::Pairing, short_weierstrass::Projective, AffineRepr, CurveGroup,
    Group,
};
use ark_ff::{BigInteger256, Field, One, UniformRand, Zero};
use ark_serialize::{CanonicalDeserialize, CanonicalSerialize};
use ark_std::test_rng;
use criterion::{BenchmarkId, Criterion};
use rand::thread_rng;
use std::ops::{Add, Div, Mul, Neg};

macro_rules! rand {
    ($typ:ty) => {{
        <$typ>::rand(&mut test_rng())
    }};
}

macro_rules! serialize {
    ($obj:expr, $method:ident) => {{
        let mut buf = vec![];
        $obj.$method(&mut buf).unwrap();
        buf
    }};
}

fn bench_group(c: &mut Criterion) {
    let mut group = c.benchmark_group("ark_bn254");

    group.bench_function("fr_add", move |b| {
        b.iter_with_setup(
            || (rand!(Fr), rand!(Fr)),
            |(k_1, k_2)| {
                let _k_3 = k_1 + k_2;
            },
        )
    });

    group.finish();
}

criterion_group!(
    name = ark_bn254_benches;
    config = Criterion::default(); //.measurement_time(Duration::from_secs(100));
    targets = bench_group);
criterion_main!(ark_bn254_benches);
