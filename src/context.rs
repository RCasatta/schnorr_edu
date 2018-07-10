use num_bigint::BigUint;
use point::Point;
use std::ops::Sub;
use std::ops::Add;
use std::ops::Div;
use std::str::FromStr;

#[allow(non_snake_case)]
pub struct Context {
    pub p: BigUint,
    pub p_sub2: BigUint,
    pub p_sub1_div2: BigUint,
    pub p_add1_div4: BigUint,
    pub two: BigUint,
    pub three: BigUint,
    pub seven: BigUint,
    pub n: BigUint,
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
