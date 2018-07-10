#[macro_use]
extern crate criterion;
extern crate rand;
extern crate num_bigint;
extern crate schnorr_edu;

use rand::prelude::*;
use criterion::Criterion;
use num_bigint::BigUint;
use std::ops::{Mul, MulAssign, Rem, Div};
use std::str::FromStr;
use schnorr_edu::*;
use schnorr_edu::point::*;
use schnorr_edu::context::*;
use schnorr_edu::biguint::*;

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
    c.bench_function("BigUint finite_sub",move |b| b.iter(|| {
        let a = rand::thread_rng().choose(&numbers).unwrap();
        let b = rand::thread_rng().choose(&numbers).unwrap();
        let c = rand::thread_rng().choose(&numbers).unwrap();
        let result = finite_sub(a.clone(),b,c);
        criterion::black_box(result);
    } ));

    let numbers = numbers_orig.clone();
    c.bench_function("BigUint mul",move |b| b.iter(|| {
        let a = (*rand::thread_rng().choose(&numbers).unwrap()).clone();
        let b = rand::thread_rng().choose(&numbers).unwrap();
        let result = a.mul(b);
        criterion::black_box(result);
    } ));

    let numbers = numbers_orig.clone();
    c.bench_function("BigUint mul assign",move |b| b.iter(|| {
        let mut a = (*rand::thread_rng().choose(&numbers).unwrap()).clone();
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
}

fn benchmark_verify(c: &mut Criterion) {
    let mut context = Context::default();
    let mut rng = thread_rng();
    let msg = [0u8;32];
    let mut sec_key = [0u8;32];
    let mut signatures = Vec::new();
    let mut pub_keys = Vec::new();
    let precomputed_signatures= 100usize;
    for _ in 0..precomputed_signatures {
        rng.fill_bytes(&mut sec_key);
        let signature = schnorr_sign(&msg,&sec_key,&context);
        let sec_key_int = BigUint::from_bytes_be(&sec_key);
        let pub_key = point_mul(Some(context.G.clone()), sec_key_int, &context) .unwrap().as_bytes();
        signatures.push(signature);
        pub_keys.push(pub_key);
    }
    context.populate_map();

    c.bench_function("Schnorr verify",move |b| b.iter(|| {
        let i = rng.gen_range(0usize, precomputed_signatures);
        let result = schnorr_verify(&msg, &pub_keys[i], &signatures[i], &context);
        criterion::black_box(result);
        assert!(result);
    } ));
}

fn benchmark_sign(c: &mut Criterion) {
    let mut rng = thread_rng();
    let mut msg = [0u8;32];
    let mut sec_key = [0u8;32];
    let mut context = Context::default();
    context.populate_map();
    c.bench_function("Schnorr sign",move |b|
        b.iter(|| {
            rng.fill_bytes(&mut msg);
            rng.fill_bytes(&mut sec_key);
            let signature = schnorr_sign(&msg, &sec_key, &context);
            criterion::black_box(signature);

        }));
}

fn benchmark_point(c: &mut Criterion) {
    let mut rng = thread_rng();
    let context = Context::default();
    let mut sec_key = [0u8;32];
    let mut points = Vec::new();
    let mut keys = Vec::new();
    let total = 100usize;
    for _ in 0..total {
        rng.fill_bytes(&mut sec_key);
        let sec_key_int = BigUint::from_bytes_be(&sec_key);
        keys.push(sec_key_int.clone());
        let point = point_mul(Some(context.G.clone()), sec_key_int, &context);
        points.push(point);
    }
    c.bench_function("EC Point multiplication",move |b|
        b.iter(|| {
            let sec_key_int = rand::thread_rng().choose(&keys).unwrap();
            let point = point_mul(Some(context.G.clone()), sec_key_int.clone(), &context);
            criterion::black_box(point);
        }));

    let context = Context::default();
    c.bench_function("EC Point adding",move |b|
        b.iter(|| {
            let a = rand::thread_rng().choose(&points).unwrap();
            let b = rand::thread_rng().choose(&points).unwrap();
            let point = point_add(a,b, &context);
            criterion::black_box(point);
        }));
}

criterion_group!{
    name = benches;
    // config = Criterion::default().sample_size(10);
    config = Criterion::default().sample_size(2).without_plots();
    targets = benchmark_biguint, benchmark_point, benchmark_verify, benchmark_sign
}

criterion_main!(benches);