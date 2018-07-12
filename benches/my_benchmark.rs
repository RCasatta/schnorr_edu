#[macro_use]
extern crate criterion;
extern crate rand;
extern crate num_bigint;
extern crate num_traits;
extern crate schnorr_edu;

use rand::prelude::*;
use criterion::Criterion;
use num_bigint::BigUint;
use std::ops::{Mul, MulAssign, Rem, Div};
use num_traits::ops::checked::CheckedSub;
use std::str::FromStr;
use schnorr_edu::*;
use schnorr_edu::point::*;
use schnorr_edu::context::*;
use schnorr_edu::scalar::*;


fn benchmark_biguint(c: &mut Criterion) {
    let mut rng = thread_rng();
    let total = 1000usize;
    let mut a = [0u8;32];
    let mut numbers_orig = Vec::new();
    for _ in 0..total {
        rng.fill_bytes(&mut a);
        numbers_orig.push( BigUint::from_bytes_be(&a) );
    }

    let numbers = numbers_orig.clone();
    c.bench_function("BigUint modpow", move|b| b.iter(|| {
        let a =  rand::thread_rng().choose(&numbers).unwrap();
        let b =  rand::thread_rng().choose(&numbers).unwrap();
        let c =  rand::thread_rng().choose(&numbers).unwrap();
        let result = a.modpow(b, c);
        criterion::black_box(result);
    } ));

    let numbers = numbers_orig.clone();
    let two = BigUint::from_str("2").unwrap();
    c.bench_function("BigUint modpow 2", move|b| b.iter(|| {
        let a =  rand::thread_rng().choose(&numbers).unwrap();
        let c =  rand::thread_rng().choose(&numbers).unwrap();
        let result = a.modpow(&two, c);
        criterion::black_box(result);
    } ));

    let numbers = numbers_orig.clone();
    c.bench_function("BigUint checked_sub",move |b| b.iter(|| {
        let a = rand::thread_rng().choose(&numbers).unwrap();
        let b = rand::thread_rng().choose(&numbers).unwrap();
        let result = a.checked_sub(b);
        criterion::black_box(result);
    } ));

    let numbers = numbers_orig.clone();
    c.bench_function("BigUint mul",move |b| b.iter(|| {
        let a = rand::thread_rng().choose(&numbers).unwrap().to_owned();
        let b = rand::thread_rng().choose(&numbers).unwrap();
        let result = a.mul(b);
        criterion::black_box(result);
    } ));

    let numbers = numbers_orig.clone();
    c.bench_function("BigUint mul assign",move |b| b.iter(|| {
        let mut a = rand::thread_rng().choose(&numbers).unwrap().to_owned();
        let b = rand::thread_rng().choose(&numbers).unwrap();
        let result = a.mul_assign(b);
        criterion::black_box(result);
    } ));

    let numbers = numbers_orig.clone();
    c.bench_function("BigUint div",move |b| b.iter(|| {
        let a = rand::thread_rng().choose(&numbers).unwrap();
        let b = rand::thread_rng().choose(&numbers).unwrap();
        let result = a.div(b);
        criterion::black_box(result);
    } ));

    let numbers = numbers_orig.clone();
    c.bench_function("BigUint rem",move |b| b.iter(|| {
        let a = rand::thread_rng().choose(&numbers).unwrap();
        let b = rand::thread_rng().choose(&numbers).unwrap();
        let result = a.rem(b);
        criterion::black_box(result);
    } ));

    let mut rng = thread_rng();
    let mut a = [0u8;32];
    c.bench_function("BigUint rand", move|b| b.iter(|| {
        rng.fill_bytes(&mut a);
        criterion::black_box(a);
    } ));

    let numbers = numbers_orig.clone();
    c.bench_function("BigUint cmp", move|b| b.iter(|| {
        let a =  rand::thread_rng().choose(&numbers).unwrap();
        let b =  rand::thread_rng().choose(&numbers).unwrap();
        let result = a>b;
        criterion::black_box(result);
    } ));
}

fn benchmark_verify(c: &mut Criterion) {
    let mut rng = thread_rng();
    let msg = [0u8;32];

    let mut signatures = Vec::new();
    let mut pub_keys = Vec::new();
    let precomputed_signatures= 100usize;
    for _ in 0..precomputed_signatures {
        let sec_key = rng.gen();
        let signature = schnorr_sign(&msg,&sec_key);
        let pub_key = point_mul(CONTEXT.G.clone(), sec_key) .unwrap();
        signatures.push(signature);
        pub_keys.push(pub_key);
    }

    c.bench_function("Schnorr verify",move |b| b.iter(|| {
        let i = rng.gen_range(0usize, precomputed_signatures);
        let result = schnorr_verify(&msg, &pub_keys[i], &signatures[i]);
        criterion::black_box(result);
        assert!(result);
    } ));
}

fn benchmark_batch_verify(c: &mut Criterion) {
    let mut rng = thread_rng();
    let mut msg = [0u8;32];
    let mut signatures = Vec::new();
    let mut pub_keys = Vec::new();
    let mut messages = Vec::new();
    let precomputed_signatures= 100usize;
    for _ in 0..precomputed_signatures {
        let sec_key = rng.gen();
        rng.fill_bytes(&mut msg);
        let signature = schnorr_sign(&msg,&sec_key);
        let pub_key = point_mul(CONTEXT.G.clone(), sec_key).unwrap();
        signatures.push(signature);
        pub_keys.push(pub_key);
        messages.push(msg);
    }


    c.bench_function("Batch verify",move |b| b.iter(|| {
        let result = schnorr_batch_verify(&messages, &pub_keys, &signatures);
        criterion::black_box(result);
        assert!(result);
    } ));
}


fn benchmark_sign(c: &mut Criterion) {
    let mut rng = thread_rng();
    let mut msg = [0u8;32];
    c.bench_function("Schnorr sign",move |b|
        b.iter(|| {
            rng.fill_bytes(&mut msg);
            let sec_key= rng.gen();
            let signature = schnorr_sign(&msg, &sec_key);
            criterion::black_box(signature);

        }));
}

fn benchmark_point(c: &mut Criterion) {
    let mut rng = thread_rng();
    let mut points = Vec::new();
    let mut keys = Vec::new();
    let total = 100usize;
    for _ in 0..total {
        let sec_key : ScalarN= rng.gen();
        let point = point_mul(CONTEXT.G.clone(), sec_key.clone()).unwrap();
        keys.push(sec_key);
        points.push(point);
    }
    c.bench_function("EC Point multiplication",move |b|
        b.iter(|| {
            let sec_key = rand::thread_rng().choose(&keys).unwrap();
            let point = point_mul(CONTEXT.G.clone(), sec_key.to_owned());
            criterion::black_box(point);
        }));

    c.bench_function("EC Point adding",move |b|
        b.iter(|| {
            let a = rand::thread_rng().choose(&points).unwrap();
            let b = rand::thread_rng().choose(&points).unwrap();
            let point = point_add(Some(a.to_owned()),Some(b.to_owned()));
            criterion::black_box(point);
        }));

    let mut points = Vec::new();
    let mut current = None;
    for _ in 0..total {
        current = jacobian_point_add(
            Some( JacobianPoint::from(CONTEXT.G.clone())),
            current);
        points.push(current.clone().unwrap());
    }
    c.bench_function("EC Jacobian Point adding",move |b|
        b.iter(|| {
            let a = rand::thread_rng().choose(&points).unwrap();
            let b = rand::thread_rng().choose(&points).unwrap();
            let point = jacobian_point_add(Some(a.to_owned()),Some(b.to_owned()));
            criterion::black_box(point);
        }));
}

criterion_group!{
    name = benches;
    // config = Criterion::default().sample_size(10);
    config = Criterion::default().sample_size(2);
    targets = benchmark_biguint, benchmark_point, benchmark_verify, benchmark_batch_verify, benchmark_sign
}

criterion_main!(benches);