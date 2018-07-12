#[macro_use]
extern crate lazy_static;

extern crate num_bigint;
extern crate num_traits;
extern crate num_integer;
extern crate data_encoding;
extern crate rand;
extern crate crypto;

pub mod point;
pub mod context;
pub mod scalar;
pub mod signature;

use std::ops::{Mul,Sub,Add};
use num_traits::One;
use num_bigint::BigUint;
use point::*;
use context::*;
use scalar::*;
use signature::*;
use rand::prelude::*;
use num_traits::Zero;

type Msg = [u8;32];

#[allow(non_snake_case)]
pub fn schnorr_sign(msg : &Msg, sec_key: &ScalarN) -> Signature {
    let sec_key_bytes = sec_key.to_32_bytes();

    let mut k = concat_and_hash(&sec_key_bytes, msg, &vec![]);
    let R = point_mul(CONTEXT.G.clone(), k.clone()).unwrap();
    if !R.y.is_jacobi() {
        k = CONTEXT.n.clone().sub(k);
    }
    let Rx = R.x.clone().to_32_bytes();
    let dG = point_mul(CONTEXT.G.clone(), (*sec_key).clone()).unwrap().as_bytes();
    let e = concat_and_hash(&Rx, &dG, msg);
    let s = k.add(e.mul((*sec_key).clone()));

    Signature::new(R.x,s)
}

#[allow(non_snake_case)]
pub fn schnorr_verify(msg : &Msg, pub_key: &Point, signature: &Signature) -> bool {
    if !pub_key.on_curve() {
        return false;
    }

    let signature_bytes = signature.as_bytes();
    let r = BigUint::from_bytes_be(&signature_bytes[..32]);
    let s = BigUint::from_bytes_be(&signature_bytes[32..]);
    if r >= CONTEXT.p.0 || s >= CONTEXT.n.0 {  // TODO Probably can't happen since ScalarN always < N
        return false;
    }
    let e = concat_and_hash(&signature_bytes[..32], &pub_key.as_bytes()[..], msg);
    let a = point_mul(CONTEXT.G.clone() , signature.s.clone());
    let b = point_mul(pub_key.to_owned() , CONTEXT.n.clone().sub(e));
    let R = point_add(a,b);

    if R.is_none() {
        return false;
    }
    let R = R.unwrap();

    if R.x != signature.Rx {
        return false;
    }

    if !R.y.is_jacobi() {
        return false
    }

    true
}

#[allow(non_snake_case)]
pub fn schnorr_batch_verify(messages : &Vec<Msg>, pub_keys:  &Vec<Point>, signatures:  &Vec<Signature>) -> bool {
    assert_eq!(messages.len(), pub_keys.len());
    assert_eq!(messages.len(), signatures.len());
    let mut R_vec= Vec::new();
    let mut a_vec= Vec::new();
    let mut e_vec= Vec::new();
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
        R_vec.push( Point{x: signature.Rx.clone(), y});
        let a = if i == 0 { ScalarN(BigUint::one()) } else { rng.gen::<ScalarN>() };
        a_vec.push(a);
    }

    let mut coeff= ScalarN(BigUint::zero());
    let mut R_point_sum = None;
    let mut P_point_sum = None;
    for i in 0..messages.len() {
        let signature = &signatures[i];
        let R = &R_vec[i];
        let a = &a_vec[i];
        let e = &e_vec[i];
        let P = &pub_keys[i];

        coeff = coeff.add( (*a).clone().mul(signature.s.clone()) );
        R_point_sum = point_add( R_point_sum, point_mul( (*R).clone(), (*a).clone()));
        P_point_sum = point_add( P_point_sum, point_mul( (*P).clone(), (*a).clone().mul((*e).clone())));
    }

    let left = CONTEXT.G.clone().mul(coeff).unwrap();
    let right = point_add( R_point_sum, P_point_sum).unwrap();

    left==right
}



#[cfg(test)]
mod tests {
    use rand::prelude::*;
    use num_bigint::BigUint;
    use super::*;
    use data_encoding::HEXUPPER;
    use context::CONTEXT;

    #[test]
    fn test_sign_and_verify() {
        let mut rng = thread_rng();
        let mut messages = Vec::new();
        let mut pub_keys = Vec::new();
        let mut signatures = Vec::new();

        let mut msg = [0u8;32];

        for _ in 0..4 {
            rng.fill_bytes(&mut msg);
            let sec_key = rng.gen::<ScalarN>();
            let pub_key = point_mul(CONTEXT.G.clone(), sec_key.clone()).unwrap();
            let signature = schnorr_sign(&msg, &sec_key);
            let result = schnorr_verify(&msg, &pub_key, &signature);
            assert!(result);

            messages.push(msg);
            pub_keys.push(pub_key);
            signatures.push(signature);
        }

        assert!(schnorr_batch_verify(&messages, &pub_keys, &signatures));
        messages.pop();
        messages.push([0u8;32]);
        assert!(!schnorr_batch_verify(&messages, &pub_keys, &signatures));
    }

    #[test]
    fn test_bip_verify() {
        fn test_vector_verify(public : &str, message : &str, signature : &str, result : bool) {
            let pub_key = Point::from_bytes( &HEXUPPER.decode(public.as_bytes()).unwrap() ).unwrap();
            let message_bytes = vec_to_32_bytes( &HEXUPPER.decode(message.as_bytes()).unwrap() );
            let signature_bytes = HEXUPPER.decode(signature.as_bytes()).unwrap();
            assert_eq!(result, schnorr_verify(&message_bytes, &pub_key, &Signature::from_bytes( &signature_bytes)));
        }
        test_vector_verify(
            "03DEFDEA4CDB677750A420FEE807EACF21EB9898AE79B9768766E4FAA04A2D4A34",
            "4DF3C3F68FCC83B27E9D42C90431A72499F17875C81A599B566C9889B9696703", "00000000000000000000003B78CE563F89A0ED9414F5AA28AD0D96D6795F9C6302A8DC32E64E86A333F20EF56EAC9BA30B7246D6D25E22ADB8C6BE1AEB08D49D",
            true
        );

        test_vector_verify(
            "02DFF1D77F2A671C5F36183726DB2341BE58FEAE1DA2DECED843240F7B502BA659",
            "243F6A8885A308D313198A2E03707344A4093822299F31D0082EFA98EC4E6C89", "2A298DACAE57395A15D0795DDBFD1DCB564DA82B0F269BC70A74F8220429BA1DFA16AEE06609280A19B67A24E1977E4697712B5FD2943914ECD5F730901B4AB7",
            false
        );

        test_vector_verify(
            "03FAC2114C2FBB091527EB7C64ECB11F8021CB45E8E7809D3C0938E4B8C0E5F84B",
            "5E2D58D8B3BCDF1ABADEC7829054F90DDA9805AAB56C77333024B9D0A508B75C", "00DA9B08172A9B6F0466A2DEFD817F2D7AB437E0D253CB5395A963866B3574BED092F9D860F1776A1F7412AD8A1EB50DACCC222BC8C0E26B2056DF2F273EFDEC",
            false
        );

        test_vector_verify(
            "03FAC2114C2FBB091527EB7C64ECB11F8021CB45E8E7809D3C0938E4B8C0E5F84B",
            "5E2D58D8B3BCDF1ABADEC7829054F90DDA9805AAB56C77333024B9D0A508B75C", "00DA9B08172A9B6F0466A2DEFD817F2D7AB437E0D253CB5395A963866B3574BED092F9D860F1776A1F7412AD8A1EB50DACCC222BC8C0E26B2056DF2F273EFDEC",
            false
        );

        test_vector_verify(
            "0279BE667EF9DCBBAC55A06295CE870B07029BFCDB2DCE28D959F2815B16F81798",
            "0000000000000000000000000000000000000000000000000000000000000000", "787A848E71043D280C50470E8E1532B2DD5D20EE912A45DBDD2BD1DFBF187EF68FCE5677CE7A623CB20011225797CE7A8DE1DC6CCD4F754A47DA6C600E59543C",
            false
        );

        test_vector_verify(
            "03DFF1D77F2A671C5F36183726DB2341BE58FEAE1DA2DECED843240F7B502BA659",
            "243F6A8885A308D313198A2E03707344A4093822299F31D0082EFA98EC4E6C89", "2A298DACAE57395A15D0795DDBFD1DCB564DA82B0F269BC70A74F8220429BA1D1E51A22CCEC35599B8F266912281F8365FFC2D035A230434A1A64DC59F7013FD",
            false
        );
    }

    #[test]
    fn test_bip_sign() {
        fn test_vector( private : &str, public : &str, message : &str, signature : &str) {
            let sec_key_bytes = vec_to_32_bytes(&HEXUPPER.decode(private.as_bytes()).unwrap());
            let sec_key = ScalarN::new(BigUint::from_bytes_be(&sec_key_bytes));
            let pub_key = point_mul(CONTEXT.G.clone(), sec_key.clone()).unwrap();
            assert_eq!(HEXUPPER.encode( &pub_key.as_bytes()[..]), public);
            let message = vec_to_32_bytes( &HEXUPPER.decode(message.as_bytes()).unwrap() );

            let signature_check =  HEXUPPER.decode(signature.as_bytes()).unwrap();
            let signature = schnorr_sign(&message, &sec_key);
            assert_eq!(signature_check, signature.as_bytes());
        }

        test_vector(
            "0000000000000000000000000000000000000000000000000000000000000001",
            "0279BE667EF9DCBBAC55A06295CE870B07029BFCDB2DCE28D959F2815B16F81798",
            "0000000000000000000000000000000000000000000000000000000000000000", "787A848E71043D280C50470E8E1532B2DD5D20EE912A45DBDD2BD1DFBF187EF67031A98831859DC34DFFEEDDA86831842CCD0079E1F92AF177F7F22CC1DCED05");

        test_vector(
            "B7E151628AED2A6ABF7158809CF4F3C762E7160F38B4DA56A784D9045190CFEF",
            "02DFF1D77F2A671C5F36183726DB2341BE58FEAE1DA2DECED843240F7B502BA659",
            "243F6A8885A308D313198A2E03707344A4093822299F31D0082EFA98EC4E6C89",
            "2A298DACAE57395A15D0795DDBFD1DCB564DA82B0F269BC70A74F8220429BA1D1E51A22CCEC35599B8F266912281F8365FFC2D035A230434A1A64DC59F7013FD");

        test_vector(
            "C90FDAA22168C234C4C6628B80DC1CD129024E088A67CC74020BBEA63B14E5C7",
            "03FAC2114C2FBB091527EB7C64ECB11F8021CB45E8E7809D3C0938E4B8C0E5F84B",
            "5E2D58D8B3BCDF1ABADEC7829054F90DDA9805AAB56C77333024B9D0A508B75C", "00DA9B08172A9B6F0466A2DEFD817F2D7AB437E0D253CB5395A963866B3574BE00880371D01766935B92D2AB4CD5C8A2A5837EC57FED7660773A05F0DE142380"
        );
    }
}
