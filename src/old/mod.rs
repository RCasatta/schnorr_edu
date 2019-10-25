use context::CONTEXT;
use point::jacobian_point::generator_mul;
use point::jacobian_point::jacobian_point_add;
use point::jacobian_point::jacobian_point_mul;
use point::jacobian_point::JacobianPoint;
use point::point::point_add;
use point::point::point_mul;
use point::Point;
use rand::thread_rng;
use rand::Rng;
use rug::Integer;
use scalar::concat_and_hash;
use scalar::ScalarN;
use std::ops::{Add, Mul, Sub};
use util::rug::integer_from_bytes;
use util::signature::Signature;
use Msg;

/// The following functions are less efficient and kept here for reference to be run by benchmarks
/// for future reference

#[allow(non_snake_case)]
pub fn schnorr_sign(msg: &Msg, sec_key: &ScalarN) -> Signature {
    let sec_key_bytes = sec_key.to_32_bytes();
    let mut k = concat_and_hash(&sec_key_bytes, msg, &vec![]);
    let R = CONTEXT.G.clone().mul(&k);
    if !R.y.jacobi() {
        k = CONTEXT.n.clone().sub(&k);
    }
    let Rx = R.x.to_32_bytes();
    let dG = CONTEXT.G.clone().mul(sec_key).as_bytes();
    let e = concat_and_hash(&Rx, &dG, msg);
    let s = k.add(e.mul(sec_key));

    Signature::new(R.x, s)
}

#[allow(non_snake_case)]
pub fn schnorr_verify(msg: &Msg, pub_key: &Point, signature: &Signature) -> bool {
    if !pub_key.on_curve() {
        return false;
    }

    let signature_bytes = signature.as_bytes();
    let r = integer_from_bytes(&signature_bytes[..32]);
    let s = integer_from_bytes(&signature_bytes[32..]);
    if r >= CONTEXT.p.0 || s >= CONTEXT.n.0 {
        // TODO Probably can't happen since ScalarN always < N
        return false;
    }
    let e = concat_and_hash(&signature_bytes[..32], &pub_key.as_bytes()[..], msg);
    let a = point_mul(CONTEXT.G.clone(), signature.s.clone());
    let b = point_mul(pub_key.to_owned(), CONTEXT.n.clone().sub(&e));
    let R = point_add(a, b);

    if R.is_none() {
        return false;
    }
    let R = R.unwrap();

    if R.x != signature.Rx {
        return false;
    }

    if !R.y.jacobi() {
        return false;
    }

    true
}

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
        let c = signature.Rx.clone().pow(&CONTEXT.three).add(&CONTEXT.seven);
        let y = c.pow(&CONTEXT.p_add1_div4);
        if y.pow(&CONTEXT.two) != c {
            return false;
        }
        R_vec.push(Point {
            x: signature.Rx.clone(),
            y,
        });
        let a = if i == 0 {
            ScalarN(Integer::from(1))
        } else {
            rng.gen::<ScalarN>()
        };
        a_vec.push(a);
    }

    let mut coeff = ScalarN(Integer::new());
    let mut R_point_sum = None;
    let mut P_point_sum = None;
    for i in 0..messages.len() {
        let signature = &signatures[i];
        let R = &R_vec[i];
        let a = &a_vec[i];
        let e = &e_vec[i];
        let P = &pub_keys[i];

        coeff = coeff.add(a.to_owned().mul(&signature.s));
        R_point_sum = point_add(point_mul(R.to_owned(), a.to_owned()), R_point_sum);
        P_point_sum = point_add(point_mul(P.to_owned(), a.to_owned().mul(e)), P_point_sum);
    }

    let left = CONTEXT.G.clone().mul(&coeff);
    let right = point_add(R_point_sum, P_point_sum).unwrap();

    left == right
}

#[allow(non_snake_case)]
pub fn schnorr_jacobi_batch_verify(
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
        let c = signature.Rx.clone().pow(&CONTEXT.three).add(&CONTEXT.seven);
        let y = c.pow(&CONTEXT.p_add1_div4);
        if y.pow(&CONTEXT.two) != c {
            return false;
        }
        R_vec.push(JacobianPoint::from(Point {
            x: signature.Rx.clone(),
            y,
        }));
        let a = if i == 0 {
            ScalarN(Integer::from(1))
        } else {
            rng.gen::<ScalarN>()
        };
        a_vec.push(a);
    }

    let mut coeff = ScalarN(Integer::new());
    let mut R_point_sum = None;
    let mut P_point_sum = None;
    //Fail if (s1 + a2s2 + ... + ausu)G ≠ R1 + a2R2 + ... + auRu + e1P1 + (a2e2)P2 + ... + (aueu)Pu
    for i in 0..messages.len() {
        let signature = &signatures[i];
        let R = &R_vec[i];
        let a = &a_vec[i];
        let e = &e_vec[i];
        let P = &pub_keys[i];

        coeff = coeff.add(a.to_owned().mul(&signature.s));
        R_point_sum = jacobian_point_add(jacobian_point_mul(R, a).as_ref(), R_point_sum.as_ref());

        let point = JacobianPoint::from(P.to_owned());
        let option = jacobian_point_mul(&point, &a.to_owned().mul(e));

        P_point_sum = jacobian_point_add(option.as_ref(), P_point_sum.as_ref());
    }

    let left = generator_mul(&coeff);
    let right = jacobian_point_add(R_point_sum.as_ref(), P_point_sum.as_ref());

    left == right
}
