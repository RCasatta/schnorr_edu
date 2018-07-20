#[macro_use]
extern crate criterion;
extern crate rand;
extern crate num_bigint;
extern crate num_traits;
extern crate schnorr_edu;
extern crate secp256k1;
extern crate apint;
extern crate rug;

use std::ops::Sub;
use rand::Rng;
use rand::thread_rng;
use rand::RngCore;
use criterion::Criterion;
use num_bigint::BigUint;
use std::ops::{Mul, MulAssign, Rem, Div};
use num_traits::ops::checked::CheckedSub;
use schnorr_edu::*;
use schnorr_edu::point::*;
use schnorr_edu::context::*;
use schnorr_edu::scalar::*;
use secp256k1::Secp256k1;
use secp256k1::Message;
use secp256k1::key::SecretKey;
use schnorr_edu::util::shamir::shamirs_trick;
use criterion::ParameterizedBenchmark;
use criterion::Throughput;
use criterion::PlotConfiguration;
use criterion::AxisScale;
use apint::UInt;
use criterion::Fun;
use criterion::Bencher;
use rug::Integer;
use rug::Assign;
use schnorr_edu::util::rug::integer_from_bytes;



#[derive(Debug)]
struct Inputs {
    biguints : Vec<BigUint>,
    apints: Vec<UInt>,
    rugs: Vec<Integer>,
}

fn benchmark_int_libraries(c: &mut Criterion) {
    let mut rng = thread_rng();
    let total = 1000usize;
    let mut apints = Vec::new();
    for _ in 0..total {
        apints.push( UInt::from_u64(rng.gen()) * &UInt::from_u64(rng.gen()) * &UInt::from_u64(rng.gen()) * &UInt::from_u64(rng.gen()) );
    }

    let mut a = [0u8;32];
    let mut biguints = Vec::new();
    for _ in 0..total {
        rng.fill_bytes(&mut a);
        biguints.push( BigUint::from_bytes_be(&a) );
    }


    let mut rugs = Vec::new();
    for _ in 0..total {
        rugs.push( Integer::from(rng.gen::<u64>()) * &Integer::from(rng.gen::<u64>())
            * &Integer::from(rng.gen::<u64>()) * &Integer::from(rng.gen::<u64>()) );
    }

    let inputs = Inputs {
        biguints, apints, rugs,
    };

    let fun_apints = Fun::new("Apints",  |b : &mut Bencher, inputs : &Inputs| b.iter(|| {
        let a =   thread_rng().choose(&inputs.apints).unwrap();
        let b =   thread_rng().choose(&inputs.apints).unwrap();
        let result = a.mul(b);
        criterion::black_box(result);
    }));

    let fun_biguints = Fun::new("BigUint",  |b : &mut Bencher , inputs: &Inputs| b.iter(|| {
        let a =   thread_rng().choose(&inputs.biguints).unwrap();
        let b =   thread_rng().choose(&inputs.biguints).unwrap();
        let result = a.mul(b);
        criterion::black_box(result);
    }));

    let fun_rugs = Fun::new("Rugs", move |b : &mut Bencher, inputs : &Inputs| b.iter(|| {
        let mut result = Integer::with_capacity(512);
        let a =   thread_rng().choose(&inputs.rugs).unwrap();
        let b =   thread_rng().choose(&inputs.rugs).unwrap();
        let incomplete_result = a.mul(b);
        result.assign(incomplete_result);
        criterion::black_box(result);
    }));

    let functions = vec!(fun_apints, fun_biguints, fun_rugs);

    c.bench_functions("256 bit mul", functions, inputs);
}


fn benchmark_rugs(c: &mut Criterion) {

}

fn benchmark_biguint(c: &mut Criterion) {
    let mut rng = thread_rng();
    let total = 1000usize;
    let mut a = [0u8;32];
    let mut numbers_orig = Vec::new();
    for _ in 0..total {
        rng.fill_bytes(&mut a);
        numbers_orig.push( integer_from_bytes(&a) );
    }

    let numbers = numbers_orig.clone();
    c.bench_function("BigUint modpow", move|b| b.iter(|| {
        let a =  rand::thread_rng().choose(&numbers).unwrap();
        let b =  rand::thread_rng().choose(&numbers).unwrap();
        let c =  rand::thread_rng().choose(&numbers).unwrap();
        let result = a.to_owned().pow_mod(b, c).unwrap();
        criterion::black_box(result);
    } ));

    let numbers = numbers_orig.clone();
    c.bench_function("ScalarP modpow one", move|b| b.iter(|| {
        let a =  ScalarP(Integer::from(1));
        let b =  rand::thread_rng().choose(&numbers).unwrap();
        let result = a.pow(&ScalarP(b.to_owned()));
        criterion::black_box(result);
    } ));

    let numbers = numbers_orig.clone();
    c.bench_function("ScalarP mul one", move|b| b.iter(|| {
        let a =  ScalarP(Integer::from(1));
        let b =  rand::thread_rng().choose(&numbers).unwrap();
        let result = a.mul(&ScalarP(b.to_owned()));
        criterion::black_box(result);
    } ));

    c.bench_function("ScalarP inv", move|b| b.iter(|| {
        let a : ScalarP =  rand::thread_rng().gen();
        criterion::black_box(a.inv());
    } ));

    let numbers = numbers_orig.clone();
    let two = Integer::from(2);
    c.bench_function("BigUint modpow 2", move|b| b.iter(|| {
        let a =  rand::thread_rng().choose(&numbers).unwrap();
        let c =  rand::thread_rng().choose(&numbers).unwrap();
        let result = a.to_owned().pow_mod(&two, c);
        criterion::black_box(result);
    } ));

    let numbers = numbers_orig.clone();
    c.bench_function("BigUint sub",move |b| b.iter(|| {
        let a = rand::thread_rng().choose(&numbers).unwrap();
        let b = rand::thread_rng().choose(&numbers).unwrap();
        let result = a.to_owned().sub(b);
        criterion::black_box(result);
    } ));

    let numbers = numbers_orig.clone();
    c.bench_function("BigUint mul",move |b| b.iter(|| {
        let a = rand::thread_rng().choose(&numbers).unwrap();
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
    let msg = [0u8;32];

    let mut signatures_orig = Vec::new();
    let mut pub_keys_orig = Vec::new();
    let precomputed_signatures= 100usize;
    for _ in 0..precomputed_signatures {
        let sec_key = thread_rng().gen();
        let signature = schnorr_sign(&msg,&sec_key);
        let pub_key = point_mul(CONTEXT.G.clone(), sec_key) .unwrap();
        signatures_orig.push(signature);
        pub_keys_orig.push(pub_key);
    }

    let signatures = signatures_orig.clone();
    let pub_keys = pub_keys_orig.clone();
    c.bench_function("Schnorr verify",move |b| b.iter(|| {
        let i = thread_rng().gen_range(0usize, precomputed_signatures);
        let result = schnorr_verify(&msg, &pub_keys[i], &signatures[i]);
        criterion::black_box(result);
        assert!(result);
    } ));

    //TOO SLOW
    /*
    let signatures = signatures_orig.clone();
    let pub_keys = pub_keys_orig.clone();
    c.bench_function("Old schnorr aff verify",move |b| b.iter(|| {
        let i = thread_rng().gen_range(0usize, precomputed_signatures);
        let result = old::schnorr_verify(&msg, &pub_keys[i], &signatures[i]);
        criterion::black_box(result);
        assert!(result);
    } ));
    */
}



fn benchmark_batch_verify(c: &mut Criterion) {
    let mut rng = thread_rng();
    let mut msg = [0u8;32];
    let mut signatures_orig = Vec::new();
    let mut pub_keys_orig = Vec::new();
    let mut messages_orig = Vec::new();
    let precomputed_signatures= 1000usize;
    for _ in 0..precomputed_signatures {
        let sec_key = rng.gen();
        rng.fill_bytes(&mut msg);
        let signature = schnorr_sign(&msg, &sec_key);
        let pub_key:Point = generator_mul(&sec_key).unwrap().into();
        signatures_orig.push(signature);
        pub_keys_orig.push(pub_key);
        messages_orig.push(msg);
    }

    // TOO SLOW
    /*
    let signatures = signatures_orig.clone();
    let pub_keys = pub_keys_orig.clone();
    let messages = messages_orig.clone();
    c.bench_function("Batch verify old ",move |b| b.iter(|| {
        let result = old::schnorr_jacobi_batch_verify(&messages, &pub_keys, &signatures);
        criterion::black_box(result);
        assert!(result);
    } ));
    */

    let signatures = signatures_orig.clone();
    let pub_keys = pub_keys_orig.clone();
    let messages = messages_orig.clone();
    let plot_config = PlotConfiguration::default()
        .summary_scale(AxisScale::Logarithmic);
    let benchmark = ParameterizedBenchmark::new("Schnorr Batch Ver", move|b, &&size|
        b.iter(||  {
            let result = schnorr_batch_verify(&messages[..size].to_vec(),
                                              &pub_keys[..size].to_vec(),
                                              &signatures[..size].to_vec());
            criterion::black_box(result);
            assert!(result);
        }), &[10, 20, 40, 80, 160, 320, 640])
        .throughput(|size| Throughput::Elements(**size as u32))
        .plot_config(plot_config);

    c.bench("Schnorr Batch Ver", benchmark);

    /*
    c.bench_function_over_inputs("Schnorr Batch Ver",move |b, &&size| b.iter(|| {
        let result = schnorr_batch_verify(&messages[..size].to_vec(),
                                          &pub_keys[..size].to_vec(),
                                          &signatures[..size].to_vec());
        criterion::black_box(result);
        assert!(result);
    } ), &[10, 100, 1000]);
    */
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

    // TOO SLOW
    /*
    let mut rng = thread_rng();
    let mut msg = [0u8;32];
    c.bench_function("Old schnorr aff sign",move |b|
        b.iter(|| {
            rng.fill_bytes(&mut msg);
            let sec_key= rng.gen();
            let signature = old::schnorr_sign(&msg, &sec_key);
            criterion::black_box(signature);
        }));
    */

    let mut rng = rand::thread_rng();
    let secp = Secp256k1::new();

    let mut msg = [0u8;32];
    c.bench_function("Schnorr libsecp sign",move |b|
        b.iter(|| {
            rng.fill_bytes(&mut msg);
            let scalar_key : ScalarN = rng.gen();
            let sk = SecretKey::from_slice(&secp, &scalar_key.to_32_bytes()).unwrap();
            let message = Message::from_slice(&msg[..]).unwrap() ;
            let signature : secp256k1::Signature = secp.sign(&message, &sk).unwrap();
            criterion::black_box(signature);
        }));
}



fn benchmark_point(c: &mut Criterion) {
    let mut points_orig = Vec::new();
    let total = 100usize;
    for _ in 0..total {
        let sec_key : ScalarN = thread_rng().gen();
        let point = point_mul(CONTEXT.G.clone(), sec_key.clone()).unwrap();
        points_orig.push(point);
    }

    let points = points_orig.clone();
    c.bench_function("EC Point adding",move |b|
        b.iter(|| {
            let a = thread_rng().choose(&points).unwrap();
            let b = thread_rng().choose(&points).unwrap();
            let point = point_add(Some(a.to_owned()),Some(b.to_owned()));
            criterion::black_box(point);
        }));


    // TOO SLOW
    /*
    let points = points_orig.clone();
    c.bench_function("EC Point multiplication",move |b|
        b.iter(|| {
            let a = thread_rng().choose(&points).unwrap();;
            let sec_key : ScalarN = thread_rng().gen();
            let point = point_mul(a.to_owned(), sec_key.to_owned());
            criterion::black_box(point);
        }));
     */

    let points_orig_affines = points_orig;

    let mut points_orig = Vec::new();
    let mut current = None;
    for _ in 0..total {
        current = jacobian_point_add(
            Some( &JacobianPoint::from(CONTEXT.G.clone())),
            current.as_ref());
        points_orig.push(current.clone().unwrap());
    }

    let points = points_orig.clone();
    c.bench_function("EC mixed Point adding",move |b|
        b.iter(|| {
            let a = thread_rng().choose(&points_orig_affines).unwrap();
            let b = thread_rng().choose(&points).unwrap();
            let point = mixed_point_add(Some(b),Some(a));
            criterion::black_box(point);
        }));


    let points = points_orig.clone();
    c.bench_function("EC Jac Point add",move |b|
        b.iter(|| {
            let a = thread_rng().choose(&points).unwrap();
            let b = thread_rng().choose(&points).unwrap();
            let point = jacobian_point_add(Some(a),Some(b));
            criterion::black_box(point);
        }));

    let points = points_orig.clone();
    c.bench_function("EC Jac Point double",move |b|
        b.iter(|| {
            let a = thread_rng().choose(&points).unwrap();;
            let point = a.to_owned().double();
            criterion::black_box(point);
        }));

    let points = points_orig.clone();
    c.bench_function("EC Jac Point mul",move |b|
        b.iter(|| {
            let sec_key : ScalarN = thread_rng().gen();
            let a = thread_rng().choose(&points).unwrap();;
            let point = jacobian_point_mul(a,&sec_key);
            criterion::black_box(point);
        }));


    let points = points_orig.clone();
    let benchmark = ParameterizedBenchmark::new("EC Jac P mul wnaf", move|b, &&size|
        b.iter(||  {
            let sec_key : ScalarN = thread_rng().gen();
            let a = thread_rng().choose(&points).unwrap();;
            let point = jacobian_point_mul_wnaf(a,&sec_key, size);
            criterion::black_box(point);
        }), &[2i8,3,4,5,6,7]);

    c.bench("EC Jac P mul wnaf", benchmark);


    c.bench_function("G JPoint mul",move |b|
        b.iter(|| {
            let sec_key : ScalarN = thread_rng().gen();
            let point = generator_mul(&sec_key).unwrap();
            criterion::black_box(point);
        }));

    let points = points_orig.clone();
    c.bench_function("EC JPoint kP+lQ",move |b|
        b.iter(|| {
            let p = thread_rng().choose(&points).unwrap();
            let q = thread_rng().choose(&points).unwrap();
            let k : ScalarN = thread_rng().gen();
            let l : ScalarN = thread_rng().gen();

            let point = jacobian_point_add(
                jacobian_point_mul(p, &k).as_ref(),
                jacobian_point_mul(q, &l).as_ref());
            criterion::black_box(point);
        }));

    let points = points_orig.clone();
    c.bench_function("EC JPoint kP+lQ shamir",move |b|
        b.iter(|| {
            let p = thread_rng().choose(&points).unwrap();
            let q = thread_rng().choose(&points).unwrap();
            let k : ScalarN = thread_rng().gen();
            let l : ScalarN = thread_rng().gen();

            let point = shamirs_trick(k,p.to_owned(),l,q.to_owned());
            criterion::black_box(point);
        }));

}

criterion_group!{
    name = benches;
    config = Criterion::default().sample_size(10);
    //config = Criterion::default().sample_size(2).without_plots();
    targets = benchmark_biguint, benchmark_point, benchmark_verify, benchmark_batch_verify, benchmark_sign, benchmark_int_libraries
}

criterion_main!(benches);