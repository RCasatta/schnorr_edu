
extern crate num_bigint;
extern crate num_traits;
extern crate num_integer;
extern crate data_encoding;
extern crate rand;
extern crate crypto;

pub mod point;
pub mod context;
pub mod biguint;
pub mod signature;

use std::ops::{Mul,Sub,Rem,Add};
use num_traits::One;
use num_bigint::BigUint;
use point::*;
use context::*;
use biguint::*;


#[allow(non_snake_case)]
pub fn schnorr_sign(msg : &[u8], sec_key: &[u8], context : &Context) -> Vec<u8> {
    let mut arg = Vec::new();
    arg.extend(sec_key);
    arg.extend(msg);
    let mut k = sha256(&arg[..]);
    let R = point_mul(Some(context.G.clone()), k.clone(), context).unwrap();
    if !jacobi(&R.y, context).is_one() {
        k = finite_sub(context.n.clone(), &k, &context.n);
    }

    let sec_key = BigUint::from_bytes_be(sec_key);
    let Rx = to_32_bytes( &R.x);
    let dG = point_mul(Some(context.G.clone()), sec_key.clone(), &context).unwrap().as_bytes();
    let mut arg = Vec::new();
    arg.extend(&Rx[..]);
    arg.extend(&dG[..]);
    arg.extend(msg);
    let e = sha256(&arg[..]);
    let s = k.add(e.mul(sec_key)).rem(&context.n);

    let mut res = Vec::new();
    res.extend(&Rx[..]);
    res.extend(&to_32_bytes(&s)[..]);

    res
}

#[allow(non_snake_case)]
pub fn schnorr_verify(msg : &[u8], pub_key_bytes: &[u8], signature: &[u8], context : &Context) -> bool {
    let pub_key = Point::from_bytes(pub_key_bytes, context).unwrap();
    if !pub_key.on_curve(context) {
        return false;
    }

    let r = BigUint::from_bytes_be(&signature[..32]);
    let s = BigUint::from_bytes_be(&signature[32..]);
    if r >= context.p || s >= context.n {
        return false;
    }
    let mut arg = Vec::new();
    arg.extend(&signature[..32]);
    arg.extend(pub_key_bytes);
    arg.extend(msg);
    let e = sha256(&arg);
    let a = point_mul(Some(context.G.clone()) , s , context);
    let b = point_mul(Some(pub_key) , context.n.clone().sub(e) , context);
    let R = point_add(&a,&b, context);

    if R.is_none() {
        return false;
    }
    let R = R.unwrap();

    if R.x != r {
        return false;
    }

    if !jacobi(&R.y, context).is_one() {
        return false
    }

    true
}




#[cfg(test)]
mod tests {
    use rand::prelude::*;
    use num_bigint::BigUint;
    use super::*;
    use data_encoding::HEXUPPER;

    #[test]
    fn test_sign_and_verify() {
        let mut rng = thread_rng();
        let mut context = Context::default();
        context.populate_map();

        let mut msg = [0u8;32];
        let mut sec_key = [0u8;32];
        for _ in 0..10 {
            rng.fill_bytes(&mut msg);
            rng.fill_bytes(&mut sec_key);
            //sec_key[31]=1;
            let sec_key_int = BigUint::from_bytes_be(&sec_key);
            let pub_key = point_mul(Some(context.G.clone()), sec_key_int, &context).unwrap().as_bytes();
            let signature = schnorr_sign(&msg, &sec_key, &context);
            let result = schnorr_verify(&msg, &pub_key, &signature, &context);
            assert!(result);
        }
    }

    #[test]
    fn test_bip_verify() {
        fn test_vector_verify(public : &str, message : &str, signature : &str, result : bool, context : &Context) {
            let public_bytes = HEXUPPER.decode(public.as_bytes()).unwrap();
            let message_bytes = HEXUPPER.decode(message.as_bytes()).unwrap();
            let signature_bytes = HEXUPPER.decode(signature.as_bytes()).unwrap();
            assert_eq!(result, schnorr_verify(&message_bytes, &public_bytes, &signature_bytes, context));
        }
        let mut context = Context::default();
        context.populate_map();
        test_vector_verify(
            "03DEFDEA4CDB677750A420FEE807EACF21EB9898AE79B9768766E4FAA04A2D4A34",
            "4DF3C3F68FCC83B27E9D42C90431A72499F17875C81A599B566C9889B9696703",
            "00000000000000000000003B78CE563F89A0ED9414F5AA28AD0D96D6795F9C6302A8DC32E64E86A333F20EF56EAC9BA30B7246D6D25E22ADB8C6BE1AEB08D49D",
            true,
            &context
        );

        test_vector_verify(
            "02DFF1D77F2A671C5F36183726DB2341BE58FEAE1DA2DECED843240F7B502BA659",
            "243F6A8885A308D313198A2E03707344A4093822299F31D0082EFA98EC4E6C89",
            "2A298DACAE57395A15D0795DDBFD1DCB564DA82B0F269BC70A74F8220429BA1DFA16AEE06609280A19B67A24E1977E4697712B5FD2943914ECD5F730901B4AB7",
            false,
            &context
        );

        test_vector_verify(
            "03FAC2114C2FBB091527EB7C64ECB11F8021CB45E8E7809D3C0938E4B8C0E5F84B",
            "5E2D58D8B3BCDF1ABADEC7829054F90DDA9805AAB56C77333024B9D0A508B75C",
            "00DA9B08172A9B6F0466A2DEFD817F2D7AB437E0D253CB5395A963866B3574BED092F9D860F1776A1F7412AD8A1EB50DACCC222BC8C0E26B2056DF2F273EFDEC",
            false,
            &context
        );

        test_vector_verify(
            "03FAC2114C2FBB091527EB7C64ECB11F8021CB45E8E7809D3C0938E4B8C0E5F84B",
            "5E2D58D8B3BCDF1ABADEC7829054F90DDA9805AAB56C77333024B9D0A508B75C",
            "00DA9B08172A9B6F0466A2DEFD817F2D7AB437E0D253CB5395A963866B3574BED092F9D860F1776A1F7412AD8A1EB50DACCC222BC8C0E26B2056DF2F273EFDEC",
            false,
            &context
        );

        test_vector_verify(
            "0279BE667EF9DCBBAC55A06295CE870B07029BFCDB2DCE28D959F2815B16F81798",
            "0000000000000000000000000000000000000000000000000000000000000000",
            "787A848E71043D280C50470E8E1532B2DD5D20EE912A45DBDD2BD1DFBF187EF68FCE5677CE7A623CB20011225797CE7A8DE1DC6CCD4F754A47DA6C600E59543C",
            false,
            &context
        );

        test_vector_verify(
            "03DFF1D77F2A671C5F36183726DB2341BE58FEAE1DA2DECED843240F7B502BA659",
            "243F6A8885A308D313198A2E03707344A4093822299F31D0082EFA98EC4E6C89",
            "2A298DACAE57395A15D0795DDBFD1DCB564DA82B0F269BC70A74F8220429BA1D1E51A22CCEC35599B8F266912281F8365FFC2D035A230434A1A64DC59F7013FD",
            false,
            &context
        );
    }

    #[test]
    fn test_bip_sign() {
        fn test_vector( private : &str, public : &str, message : &str, signature : &str, context : &Context) {
            let sec_key_bytes = HEXUPPER.decode(private.as_bytes()).unwrap();
            let sec_key = BigUint::from_bytes_be(&sec_key_bytes);
            let pub_key = point_mul(Some(context.G.clone()), sec_key, &context).unwrap();
            assert_eq!(HEXUPPER.encode( &pub_key.as_bytes()[..]), public);
            let message = HEXUPPER.decode(message.as_bytes()).unwrap();
            let signature_check =  HEXUPPER.decode(signature.as_bytes()).unwrap();
            let signature = schnorr_sign(&message, &sec_key_bytes, &context);
            assert_eq!(signature_check, signature);
        }
        let mut context = Context::default();
        context.populate_map();
        test_vector(
            "0000000000000000000000000000000000000000000000000000000000000001",
            "0279BE667EF9DCBBAC55A06295CE870B07029BFCDB2DCE28D959F2815B16F81798",
            "0000000000000000000000000000000000000000000000000000000000000000",
            "787A848E71043D280C50470E8E1532B2DD5D20EE912A45DBDD2BD1DFBF187EF67031A98831859DC34DFFEEDDA86831842CCD0079E1F92AF177F7F22CC1DCED05",
            &context
        );

        test_vector(
            "B7E151628AED2A6ABF7158809CF4F3C762E7160F38B4DA56A784D9045190CFEF",
            "02DFF1D77F2A671C5F36183726DB2341BE58FEAE1DA2DECED843240F7B502BA659",
            "243F6A8885A308D313198A2E03707344A4093822299F31D0082EFA98EC4E6C89",
            "2A298DACAE57395A15D0795DDBFD1DCB564DA82B0F269BC70A74F8220429BA1D1E51A22CCEC35599B8F266912281F8365FFC2D035A230434A1A64DC59F7013FD",
            &context
        );

        test_vector(
            "C90FDAA22168C234C4C6628B80DC1CD129024E088A67CC74020BBEA63B14E5C7",
            "03FAC2114C2FBB091527EB7C64ECB11F8021CB45E8E7809D3C0938E4B8C0E5F84B",
            "5E2D58D8B3BCDF1ABADEC7829054F90DDA9805AAB56C77333024B9D0A508B75C",
            "00DA9B08172A9B6F0466A2DEFD817F2D7AB437E0D253CB5395A963866B3574BE00880371D01766935B92D2AB4CD5C8A2A5837EC57FED7660773A05F0DE142380",
            &context
        );
    }

}
