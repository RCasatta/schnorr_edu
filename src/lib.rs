
extern crate num_bigint;
extern crate num_traits;
extern crate num_integer;
extern crate data_encoding;
extern crate rand;
extern crate crypto;

use std::ops::{Mul,Sub,Div,Rem,Add};
use num_traits::{Zero, One};
use num_bigint::BigUint;
use num_integer::Integer;
use std::str::FromStr;
use crypto::sha2::Sha256;
use crypto::digest::Digest;
use std::fmt;

#[derive(Clone, PartialEq, Eq)]
pub struct Point {
    x: BigUint,
    y: BigUint,
}

#[allow(non_snake_case)]
pub struct Context {
    p: BigUint,
    p_sub2: BigUint,
    p_sub1_div2: BigUint,
    p_add1_div4: BigUint,
    two: BigUint,
    three: BigUint,
    seven: BigUint,
    n: BigUint,
    pub G: Point,
}

impl Default for Context {
    fn default() -> Self {
        //TODO lazy_static?
        let p = BigUint::parse_bytes("FFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFEFFFFFC2F".as_bytes(),16).unwrap();
        let p_sub1 = p.clone().sub(1u32);
        let p_add1 = p.clone().add(1u32);
        Context {
            p : p.clone(),
            p_sub2 : p.clone().sub(2u32),
            p_sub1_div2 : p_sub1.div(2u32),
            p_add1_div4: p_add1.div(4u32),
            two: BigUint::from_str("2").unwrap(),
            three: BigUint::from_str("3").unwrap(),
            seven: BigUint::from_str("7").unwrap(),
            n : BigUint::parse_bytes("FFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFEBAAEDCE6AF48A03BBFD25E8CD0364141".as_bytes(),16).unwrap(),
            G : Point {
                x: BigUint::parse_bytes("79BE667EF9DCBBAC55A06295CE870B07029BFCDB2DCE28D959F2815B16F81798".as_bytes(),16).unwrap(),
                y: BigUint::parse_bytes("483ADA7726A3C4655DA4FBFC0E1108A8FD17B448A68554199C47D08FFB10D4B8".as_bytes(),16).unwrap(),
            }
        }
    }
}

pub fn point_mul(mut p: Option<Point>, mut n : BigUint, context : &Context) -> Option<Point> {
    let mut r : Option<Point> = None;

    loop {
        let (ris, rem) = n.div_rem(&context.two);

        if rem.is_one() {
            r = point_add(&r,&p, context);
        }
        p = point_add(&p,&p, context);
        n = ris.clone();
        if ris.is_zero() {
            return r;
        }
    }
}

fn point_add(p1 : &Option<Point>, p2 : &Option<Point>, context : &Context) -> Option<Point> {
    match (p1,p2) {
        (None, None) => None,
        (Some(p1), None) => Some(p1.clone()),
        (None, Some(p2)) => Some(p2.clone()),
        (Some(p1), Some(p2)) => {
            if  p1.x == p2.x && p1.y != p2.y {
                return None;
            }
            let lam = if  p1 == p2 {
                // lam = (3 * p1[0] * p1[0] * pow(2 * p1[1], p - 2, p)) % p
                let pow = p1.y.clone().mul(2u32).modpow(&context.p_sub2, &context.p);
                context.three.clone().mul(&p1.x).rem(&context.p).mul(&p1.x).rem(&context.p).mul(&pow).rem(&context.p)
            } else {
                // lam = ((p2[1] - p1[1]) * pow(p2[0] - p1[0], p - 2, p)) % p
                let pow = finite_sub( p2.x.clone(), p1.x.clone(), context.p.clone()).modpow(&context.p_sub2, &context.p);
                finite_sub( p2.y.clone(), p1.y.clone(), context.p.clone() ).mul(pow).rem(&context.p)
            };
            // x3 = (lam * lam - p1[0] - p2[0]) % p
            let x3 = lam.clone().modpow(&context.two, &context.p);
            let x3 = finite_sub(x3,p1.x.clone(),context.p.clone());
            let x3 = finite_sub(x3,p2.x.clone(),context.p.clone());

            //(x3, (lam * (p1[0] - x3) - p1[1]) % p)
            let sub = finite_sub(p1.x.clone(), x3.clone(), context.p.clone());
            let mut y3 = lam.mul(sub).sub(&p1.y).rem(&context.p);  // check neg

            Some(Point{x:x3,y:y3})
        }
    }
}

fn finite_sub(a : BigUint, b : BigUint, p_or_n : BigUint) -> BigUint{
    if a > b {
        a.sub(b)
    } else {
        finite_sub(a.add(p_or_n.clone()), b, p_or_n)
    }
}

#[allow(non_snake_case)]
pub fn schnorr_sign(msg : &[u8], sec_key: &[u8], context : &Context) -> Vec<u8> {
    let mut arg = Vec::new();
    arg.extend(sec_key);
    arg.extend(msg);
    let mut k = sha256(&arg[..]);
    let R = point_mul(Some(context.G.clone()), k.clone(), context).unwrap();
    if !jacobi(&R.y, context).is_one() {
        k = finite_sub(context.n.clone(), k.clone(), context.n.clone());
    }

    let sec_key = BigUint::from_bytes_be(sec_key);
    let rx = to_32_bytes( &R.x);
    let dG = point_mul(Some(context.G.clone()), sec_key.clone(), &context).unwrap().as_bytes();
    let mut arg = Vec::new();
    arg.extend(&rx[..]);
    arg.extend(&dG[..]);
    arg.extend(msg);
    let e = sha256(&arg[..]);
    let s = k.add(e.mul(sec_key)).rem(&context.n);

    let mut res = Vec::new();
    res.extend(&to_32_bytes(&R.x)[..]);
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

pub fn sha256(input : &[u8]) -> BigUint {
    let mut hashed = [0u8;32];
    let mut hasher = Sha256::new();
    hasher.input(input);
    hasher.result(&mut hashed);
    BigUint::from_bytes_be( &hashed[..])
}

pub fn jacobi(x : &BigUint, context : &Context) -> BigUint {
    x.modpow(&context.p_sub1_div2,&context.p)
}

fn to_32_bytes(val : &BigUint) -> [u8;32] {
    let bytes = val.to_bytes_be();
    let mut result = [0u8;32];
    let start = 32-bytes.len();
    assert!(start<=32);
    for i in start..32usize {
        result[i]=bytes[i-start];
    }
    result
}

impl fmt::Display for Point {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "({}, {})", self.x, self.y)
    }
}

impl Point {

    pub fn on_curve(&self, context : &Context) -> bool {
        let pow1 = self.y.modpow(&context.two, &context.p);
        let pow2 = self.x.modpow(&context.three, &context.p);
        let sub = finite_sub(pow1, pow2, context.p.clone());

        sub.rem(&context.p) == context.seven
    }

    pub fn as_bytes(&self) -> [u8;33] {
        let mut res = [0u8;33];
        if self.y.clone().rem(2u32).is_zero() {
            res[0] = 0x02;
        } else {
            res[0] = 0x03;
        }
        let bytes = to_32_bytes(&self.x);
        res[1..].copy_from_slice(&bytes[..]);
        res
    }

    pub fn from_bytes(bytes : &[u8], context : &Context) -> Option<Self> {
        if bytes.len()!=33 {
            return None;
        }
        let x =  BigUint::from_bytes_be(&bytes[1..]);
        let y2 = x.modpow(&context.three, &context.p).add(&context.seven);

        // in secp256k1 sqrt is equal to pow( (p-1)/4 )
        let mut y : BigUint = y2.modpow(&context.p_add1_div4, &context.p);
        if ( bytes[0]==0x02 && y.is_odd() ) || ( bytes[0]==0x03 && y.is_even() ) {
            y = context.p.clone().sub(y);
        }
        Some(Point {x,y})
    }
}


#[cfg(test)]
mod tests {

    use num_bigint::BigUint;
    use super::*;
    use data_encoding::HEXLOWER;
    use data_encoding::HEXUPPER;

    #[test]
    fn text_context_mul_and_add() {
        let context = Context::default();
        assert_eq!("115792089237316195423570985008687907853269984665640564039457584007908834671663", format!("{}", context.p));
        assert_eq!("55066263022277343669578718895168534326250603453777594175500187360389116729240", format!("{}", context.G.x));
        assert_eq!("32670510020758816978083085130507043184471273380659243275938904335757337482424", format!("{}", context.G.y));

        let g2 = point_add(&Some(context.G.clone()), &Some(context.G.clone()), &context).unwrap();
        assert_eq!("89565891926547004231252920425935692360644145829622209833684329913297188986597", format!("{}", g2.x));
        assert_eq!("12158399299693830322967808612713398636155367887041628176798871954788371653930", format!("{}", g2.y));

        let g3 = point_add(&Some(context.G.clone()), &Some(g2.clone()), &context).unwrap();
        assert_eq!("112711660439710606056748659173929673102114977341539408544630613555209775888121", format!("{}", g3.x));
        assert_eq!("25583027980570883691656905877401976406448868254816295069919888960541586679410", format!("{}", g3.y));

        let g2b = point_mul(Some(context.G.clone()), context.two.clone(), &context).unwrap();
        assert_eq!(g2.x,g2b.x);
        assert_eq!(g2.y,g2b.y);

        let g3b = point_mul(Some(context.G.clone()), context.three.clone(), &context).unwrap();
        assert_eq!(g3.x,g3b.x);
        assert_eq!(g3.y,g3b.y);

        let g8675309 = point_mul(Some(context.G.clone()), BigUint::from_str("8675309").unwrap(), &context).unwrap();
        assert_eq!("66641067246008511739397675128206923493293851901978595085468284019495272794983", format!("{}", g8675309.x));
        assert_eq!("22882405661336615738255795181502754819791112635438119114432507482219105379189", format!("{}", g8675309.y));
    }



    #[test]
    fn test_point() {
        let context = Context::default();
        let x_bytes = context.G.as_bytes();
        let x = HEXLOWER.encode(&x_bytes[..]);
        assert_eq!(x,"0279be667ef9dcbbac55a06295ce870b07029bfcdb2dce28d959f2815b16f81798");

        assert!(context.G.on_curve(&context));

        let g_deserialized = Point::from_bytes(&x_bytes, &context).unwrap();
        assert_eq!(&context.G.x, &g_deserialized.x);
        assert_eq!(&context.G.y, &g_deserialized.y);
    }

    #[test]
    fn test_sign_and_verify() {
        let context = Context::default();
        let msg = [0u8;32];
        let sec_key = [1u8;32];
        //sec_key[31]=1;
        let sec_key_int = BigUint::from_bytes_be(&sec_key);
        let pub_key = point_mul(Some(context.G.clone()), sec_key_int, &context) .unwrap().as_bytes();
        let signature = schnorr_sign(&msg,&sec_key,&context);
        let result = schnorr_verify(&msg, &pub_key, &signature, &context);
        assert!(result);
    }

    #[test]
    fn test_bip_verify() {
        fn test_vector_verify(public : &str, message : &str, signature : &str, result : bool, context : &Context) {
            let public_bytes = HEXUPPER.decode(public.as_bytes()).unwrap();
            let message_bytes = HEXUPPER.decode(message.as_bytes()).unwrap();
            let signature_bytes = HEXUPPER.decode(signature.as_bytes()).unwrap();
            assert_eq!(result, schnorr_verify(&message_bytes, &public_bytes, &signature_bytes, context));
        }
        let context = Context::default();
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
        let context = Context::default();
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
