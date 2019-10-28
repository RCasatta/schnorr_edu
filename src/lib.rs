#[macro_use]
extern crate lazy_static;

extern crate apint;
extern crate crypto;
extern crate data_encoding;
extern crate num_bigint;
extern crate num_integer;
extern crate num_traits;
extern crate rand;
extern crate rug;

pub mod context;
pub mod old;
pub mod point;
pub mod scalar;
pub mod util;

use context::CONTEXT;
use point::{generator_mul, jacobian_point_add};
use point::{JacobianPoint, Point};
use rand::thread_rng;
use rand::Rng;
use rug::Integer;
use scalar::concat_and_hash_BIPSchnorrDerive;
use scalar::concat_and_hash_BIPSchnorr;
use scalar::concat_and_hash;
use scalar::ScalarN;
use std::borrow::Borrow;
use std::collections::BinaryHeap;
use std::ops::{Add, Mul, Sub};
use util::rug::integer_from_bytes;
use util::signature::Signature;
use util::term::Term;
use data_encoding::HEXUPPER;

type Msg = [u8; 32];

// https://github.com/sipa/bips/blob/bip-schnorr/bip-schnorr.mediawiki

#[allow(non_snake_case)]
pub fn schnorr_sign(msg: &Msg, sec_key: &ScalarN) -> Signature {
    println!("schnorr_sign(msg, sec_key) with ({},{})", &HEXUPPER.encode(msg), &HEXUPPER.encode(&sec_key.to_32_bytes()));
    let sec_key_bytes = sec_key.to_32_bytes();
    let P_jacobian = generator_mul(&sec_key).unwrap();
    let P = Point::from(P_jacobian);

    println!("schnorr_sign  P.y.is_square()? {}",  P.y.is_square());
    let sec_key_sq = if P.y.is_square() {
        sec_key.clone()
    } else {
        CONTEXT.n.clone().sub(sec_key)
    };

    let mut k0 = concat_and_hash_BIPSchnorrDerive(&sec_key_sq.to_32_bytes(), msg, &vec![]);
    let R_jacobian = generator_mul(&k0).unwrap();

    let R = Point::from(R_jacobian);
    println!("schnorr_sign  R.y.is_square()? {}",  R.y.is_square());
    let k = if R.y.is_square() {
        k0
    } else {
        CONTEXT.n.clone().sub(&k0)
    };
    //println!("sign k={}", k);

    let e = concat_and_hash_BIPSchnorr(&R.as_bytes(), &P.as_bytes(), msg);
    //println!("sign e={}", e);

    let s = k.add(e.mul(&sec_key_sq));

    Signature::new(R.x, s)
}

#[allow(non_snake_case)]
pub fn schnorr_verify(msg: &Msg, pub_key: &Point, signature: &Signature) -> bool {
    println!("schnorr_verify(msg, pub_key, signature) with ({},{},{})", &HEXUPPER.encode(&msg[..]), &HEXUPPER.encode(&pub_key.as_bytes()), &HEXUPPER.encode(&signature.as_bytes()));

    if !pub_key.on_curve() {
        return false;
    }

    let signature_bytes = signature.as_bytes();
    let r = integer_from_bytes(&signature_bytes[..32]);
    let s = integer_from_bytes(&signature_bytes[32..]);
    if r >= CONTEXT.p.0 || s >= CONTEXT.n.0 {
        return false;
    }
    let e = concat_and_hash_BIPSchnorr(&signature_bytes[..32], &pub_key.as_bytes()[..], msg);
    //println!("e {}", e);

    let a = generator_mul(&signature.s).unwrap();
    let b = JacobianPoint::from(pub_key.to_owned()).mul(&CONTEXT.n.clone().sub(&e));
    let R = jacobian_point_add(Some(&a), Some(&b));
    if R.is_none() {
        println!("3");

        return false;
    }
    let R = R.unwrap();

    let R = Point::from(R);
    if !R.y.is_square() {
        println!("4");
        return false;
    }

    // x(P) ≠ r can be implemented as x ≠ z^2r mod p.
    //let Rx = R.z.clone().mul(&R.z).mul(&signature.Rx);
    if R.x != signature.Rx {
        println!("5");
        return false;
    }

    true
}

// https://www.deadalnix.me/2017/02/17/schnorr-signatures-for-not-so-dummies/
#[allow(non_snake_case)]
pub fn schnorr_batch_verify(
    messages: &Vec<Msg>,
    pub_keys: &Vec<Point>,
    signatures: &Vec<Signature>,
) -> bool {
    assert_eq!(messages.len(), pub_keys.len());
    assert_eq!(messages.len(), signatures.len());
    let mut R_vec = Vec::new();
    let mut a_vec = Vec::new();
    let mut e_vec = Vec::new();
    let mut rng = thread_rng();
    for i in 0..messages.len() {
        let msg = &messages[i];
        let P = &pub_keys[i];
        let signature = &signatures[i];
        if !P.on_curve() {
            return false;
        }
        let e = concat_and_hash(&signature.Rx.to_32_bytes(), &P.as_bytes(), &msg[..]);
        e_vec.push(e);
        let c = signature
            .Rx
            .borrow()
            .pow(&CONTEXT.three)
            .add(&CONTEXT.seven);
        let y = c.pow(&CONTEXT.p_add1_div4);
        let y_pow2 = y.clone().mul(&y);
        if y_pow2 != c {
            return false;
        }
        R_vec.push(JacobianPoint::from(Point {
            x: signature.Rx.clone(),
            y,
        }));
        let a = if i == 0 {
            ScalarN(Integer::from(1u32))
        } else {
            rng.gen::<ScalarN>()
        };
        a_vec.push(a);
    }

    //Fail if (s1 + a2s2 + ... + ausu)G ≠ R1 + a2R2 + ... + auRu + e1P1 + (a2e2)P2 + ... + (aueu)Pu
    let mut coeff = ScalarN(Integer::new());
    let mut inner_product: Vec<Term> = Vec::new();
    for i in 0..messages.len() {
        let signature = &signatures[i];
        let R = &R_vec[i];
        let a = &a_vec[i];
        let e = &e_vec[i];
        let P = &pub_keys[i];

        coeff = coeff.add(a.to_owned().mul(&signature.s));
        inner_product.push(Term {
            coeff: a.to_owned(),
            point: R.to_owned(),
        });
        inner_product.push(Term {
            coeff: a.to_owned().mul(e),
            point: JacobianPoint::from(P.to_owned()),
        });
    }
    inner_product.push(Term {
        coeff: CONTEXT.n.clone().sub(&coeff),
        point: CONTEXT.G_jacobian.clone(),
    }); // -sG

    let mut inner_product: BinaryHeap<Term> = BinaryHeap::from(inner_product);

    while inner_product.len() > 1 {
        let t0 = inner_product.pop().unwrap();
        let t1 = inner_product.pop().unwrap();
        let option = jacobian_point_add(Some(&t0.point), Some(&t1.point));
        if option.is_none() && inner_product.len() == 0 {
            return true;
        }
        inner_product.push(Term {
            coeff: t1.coeff.clone(),
            point: option.unwrap(),
        });
        if t0.coeff != t1.coeff {
            inner_product.push(Term {
                coeff: t0.coeff - &t1.coeff,
                point: t0.point,
            });
        }
    }
    return false;
}

#[cfg(test)]
mod tests {
    use super::*;
    use data_encoding::HEXUPPER;
    use old;
    use rand::prelude::*;
    use scalar::vec_to_32_bytes;

    #[test]
    fn test_sign_and_jacobi_sign() {
        let sec_key = thread_rng().gen::<ScalarN>();
        let msg = [0u8; 32];
        let sign1 = schnorr_sign(&msg, &sec_key);
        let sign2 = schnorr_sign(&msg, &sec_key);
        assert_eq!(sign1, sign2);
    }


    #[test]
    fn test_sign_and_verify() {
        let mut rng = thread_rng();
        let mut messages = Vec::new();
        let mut pub_keys = Vec::new();
        let mut signatures = Vec::new();

        let mut msg = [0u8; 32];

        for i in 0..10 {
            println!("{}", i);
            rng.fill_bytes(&mut msg);
            let sec_key = rng.gen::<ScalarN>();
            let pub_key: Point = generator_mul(&sec_key).unwrap().into();
            let signature = schnorr_sign(&msg, &sec_key);
            let result = schnorr_verify(&msg, &Point::from_bytes(&pub_key.as_bytes()).unwrap(), &signature);
            println!("result {}", result);
            assert!(result);
            /*
            if i == 0 {
                // too slow to do it every time
                let result = old::schnorr_verify(&msg, &pub_key, &signature);
                assert!(result);
            }
            */

            messages.push(msg);
            pub_keys.push(pub_key);
            signatures.push(signature);
        }
        /*
        assert!(schnorr_batch_verify(&messages, &pub_keys, &signatures));
        assert!(old::schnorr_batch_verify(
            &messages[..2].to_vec(),
            &pub_keys[..2].to_vec(),
            &signatures[..2].to_vec()
        ));
        messages.pop();
        messages.push([0u8; 32]);
        assert!(!schnorr_batch_verify(&messages, &pub_keys, &signatures));
        */
    }

    #[test]
    fn test_new_bip_verify() {
        let test_vectors = include_str!("../test-vectors.csv");

        for (i,line) in test_vectors.lines().enumerate() {
            if i == 0 {
                continue;
            }
            let mut cols = line.split(",");
            let index = cols.next().unwrap();
            let secret_key = cols.next().unwrap();
            let public_key = cols.next().unwrap();
            let message = cols.next().unwrap();
            let signature = cols.next().unwrap();
            let result = cols.next().unwrap();
            let comment = cols.next().unwrap();

            if !secret_key.is_empty() {
                test_vector(&secret_key, &public_key, &message, &signature, "TRUE" == result);
            }
            test_vector_verify(&public_key, &message, &signature, "TRUE" == result);
        }

    }

    fn test_vector_verify(public: &str, message: &str, signature: &str, result: bool) {
        println!("{} {} {} {}", public, message, signature, result);
        let pub_key = Point::from_bytes(&HEXUPPER.decode(public.as_bytes()).unwrap()).unwrap();
        let message_bytes = vec_to_32_bytes(&HEXUPPER.decode(message.as_bytes()).unwrap());
        let signature_bytes = HEXUPPER.decode(signature.as_bytes()).unwrap();
        let signature_result = Signature::from_bytes(&signature_bytes);
        if signature_result.is_ok() {
            assert_eq!(
                result,
                schnorr_verify(
                    &message_bytes,
                    &pub_key,
                    &signature_result.unwrap()
                )
            );
        } else {
            println!("ERRRRRRR");
        }
    }

    fn test_vector(private: &str, public: &str, message: &str, signature: &str, result: bool) {
        println!("{} {} {} {} {}", private, public, message, signature, result);

        let sec_key_bytes = vec_to_32_bytes(&HEXUPPER.decode(private.as_bytes()).unwrap());
        let sec_key = ScalarN::new(integer_from_bytes(&sec_key_bytes));
        let pub_key: Point = generator_mul(&sec_key).unwrap().into();
        assert_eq!(HEXUPPER.encode(&pub_key.as_bytes()[..]), public);
        let message = vec_to_32_bytes(&HEXUPPER.decode(message.as_bytes()).unwrap());

        let signature_check = HEXUPPER.decode(signature.as_bytes()).unwrap();
        let signature = schnorr_sign(&message, &sec_key);
        assert_eq!(signature_check, signature.as_bytes());
    }

}
