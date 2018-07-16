
use num_bigint::BigUint;
use point::Point;
use std::ops::Sub;
use std::ops::Add;
use std::ops::Div;
use data_encoding::HEXLOWER;
use scalar::ScalarN;
use scalar::ScalarP;
use num_traits::One;
use std::str::FromStr;
use point::JacobianPoint;
use std::fs::File;
use std::io::Read;

lazy_static! {
    pub static ref CONTEXT: Context = {
        Context::default()
    };
}

#[allow(non_snake_case)]
pub struct Context {
    pub p: ScalarP,
    pub p_sub2: ScalarP,
    pub p_sub1_div2: ScalarP,
    pub p_add1_div4: ScalarP,
    pub two: ScalarP,
    pub three: ScalarP,
    pub four: ScalarP,
    pub seven: ScalarP,
    pub eight: ScalarP,
    pub n: ScalarN,
    pub G: Point,
    pub G_jacobian: JacobianPoint,
}

impl Default for Context {
    fn default() -> Self {
        let p = BigUint::parse_bytes("FFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFEFFFFFC2F".as_bytes(),16).unwrap();
        let one = BigUint::one();
        let two = BigUint::from_str("2").unwrap();
        let three =  BigUint::from_str("3").unwrap();
        let four = BigUint::from_str("4").unwrap();
        let seven = BigUint::from_str("7").unwrap();
        let eight = BigUint::from_str("8").unwrap();
        let p_sub1 = p.clone().sub(&one);
        let p_add1 = p.clone().add(&one);
        let g = Point {
            x: ScalarP(BigUint::parse_bytes("79BE667EF9DCBBAC55A06295CE870B07029BFCDB2DCE28D959F2815B16F81798".as_bytes(), 16).unwrap()),
            y: ScalarP(BigUint::parse_bytes("483ADA7726A3C4655DA4FBFC0E1108A8FD17B448A68554199C47D08FFB10D4B8".as_bytes(), 16).unwrap()),
        };


        Context {
            p : ScalarP(p.clone()),
            p_sub2 : ScalarP(p.clone().sub(&two)),
            p_sub1_div2 : ScalarP(p_sub1.div(&two)),
            p_add1_div4: ScalarP(p_add1.div(&four)),
            two: ScalarP(two),
            three: ScalarP(three),
            four: ScalarP(four),
            seven: ScalarP(seven),
            eight: ScalarP(eight),
            n : ScalarN(BigUint::parse_bytes("FFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFEBAAEDCE6AF48A03BBFD25E8CD0364141".as_bytes(),16).unwrap()),
            G : g.clone(),
            G_jacobian: JacobianPoint::from(g),
        }
    }
}

lazy_static! {
    pub static ref AFFINES_DOUBLES_CACHE: Vec<Point> = {  // I can't initialize [Point;256]
        let mut cache = Vec::with_capacity(256);
        let lines = include_str!("doubles_cache.in").lines();
        for line in lines {
            cache.push(Point::from_bytes(&HEXLOWER.decode(line.as_bytes()).unwrap()).unwrap());
        }
        cache
    };
}

lazy_static! {
    pub static ref JACOBIAN_DOUBLES_CACHE: Vec<JacobianPoint> = {
        let mut cache = Vec::with_capacity(256);
        let lines = include_str!("doubles_cache.in").lines();
        for line in lines {
            cache.push(JacobianPoint::from(Point::from_bytes(&HEXLOWER.decode(line.as_bytes()).unwrap()).unwrap()));
        }
        cache
    };
}

lazy_static! {
    pub static ref BIG_CACHE: Vec<JacobianPoint> = {
        let mut vec = Vec::with_capacity(8160);
        let mut f = File::open("big_cache.dat").unwrap();
        let mut buffer = [0; 33];
        for _ in 0..8160 {
            f.read(&mut buffer).unwrap();
            vec.push(JacobianPoint::from_bytes(&buffer).unwrap());
        }
        vec
    };
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_lazy_static() {
        let context = Context::default();
        assert_eq!(CONTEXT.G , context.G);

    }

    #[test]
    fn test_load_big() {
        let option = BIG_CACHE.get(0);

    }
}