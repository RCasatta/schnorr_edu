
use point::Point;
use scalar::ScalarN;
use scalar::ScalarP;
use point::JacobianPoint;
use std::fs::File;
use std::io::Read;
use rug::Integer;
use std::borrow::Borrow;
use std::ops::Sub;
use std::ops::Add;
use std::ops::Div;

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

        let p = Integer::from_str_radix("FFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFEFFFFFC2F",16).unwrap();
        let one = Integer::from(1u8);
        let two = Integer::from(2u8);
        let three =  Integer::from(3u8);
        let four = Integer::from(4u8);
        let seven = Integer::from(7u8);
        let eight = Integer::from(8u8);

        let p_sub1 : Integer = p.borrow().sub(&one).into();
        let p_add1 : Integer = p.clone().add(&one).into();
        let g = Point {
            x: ScalarP(Integer::from_str_radix("79BE667EF9DCBBAC55A06295CE870B07029BFCDB2DCE28D959F2815B16F81798", 16).unwrap()),
            y: ScalarP(Integer::from_str_radix("483ADA7726A3C4655DA4FBFC0E1108A8FD17B448A68554199C47D08FFB10D4B8", 16).unwrap()),
        };


        Context {
            p : ScalarP(p.clone()),
            p_sub2 : ScalarP(p.clone().sub(&two)),
            p_sub1_div2 : ScalarP(p_sub1.clone().div(&two)),
            p_add1_div4: ScalarP(p_add1.clone().div(&four)),
            two: ScalarP(two),
            three: ScalarP(three),
            four: ScalarP(four),
            seven: ScalarP(seven),
            eight: ScalarP(eight),
            n : ScalarN(Integer::from_str_radix("FFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFEBAAEDCE6AF48A03BBFD25E8CD0364141",16).unwrap()),
            G : g.clone(),
            G_jacobian: JacobianPoint::from(g),
        }
    }
}

lazy_static! {
    pub static ref G_MUL_CACHE: Vec<Point> = {
        let total_elements=8192usize;
        let mut vec = Vec::with_capacity(total_elements);
        let mut f = File::open("res/g_mul_cache.dat").unwrap();
        let mut buffer = [0; 64];
        for _ in 0..total_elements {
            f.read(&mut buffer).unwrap();
            let option = Point::from_uncompressed_bytes(&buffer);
            vec.push(option.unwrap());
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
        let option = G_MUL_CACHE.get(0).unwrap();
        assert_eq!(CONTEXT.G , option.to_owned());


    }
}