#[macro_use]
extern crate criterion;
extern crate rand;
extern crate num_bigint;
extern crate schnorr_edu;

use rand::prelude::*;
use criterion::Criterion;
use schnorr_edu::*;
use num_bigint::BigUint;

fn benchmark_verify(c: &mut Criterion) {
    let context = Context::default();
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

    c.bench_function("Schnorr verify",move |b| b.iter(|| {
        let i = rng.gen_range(0usize, precomputed_signatures);
        let result = schnorr_verify(&msg, &pub_keys[i], &signatures[i], &context);
        assert!(result);
    } ));
}

fn benchmark_sign(c: &mut Criterion) {
    let mut rng = thread_rng();
    let mut msg = [0u8;32];
    let mut sec_key = [0u8;32];
    let context = Context::default();
    c.bench_function("Schnorr sign",move |b|
        b.iter(|| {
            rng.fill_bytes(&mut msg);
            rng.fill_bytes(&mut sec_key);
        schnorr_sign(&msg,&sec_key,&context);
        } ));


}

criterion_group!{
    name = benches;
    config = Criterion::default().sample_size(2);
    targets = benchmark_verify, benchmark_sign
}

criterion_main!(benches);