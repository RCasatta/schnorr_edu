use scalar::ScalarP;
use scalar::ScalarN;
use point::Point;
use num_bigint::BigUint;
use num_traits::One;
use context::CONTEXT;
use context::G_MUL_CACHE;
use std::ops::{Mul,Sub,Add};
use std::fmt;
use num_traits::Zero;
use std::borrow::Borrow;

// Very bad defining Eq like this since two equal Jacobian Point could have different coordinates
// however it's useful for now and used only in the HashMap where values are normalized
#[derive(Clone, Debug)]
pub struct JacobianPoint {
    pub x: ScalarP,
    pub y: ScalarP,
    pub z: ScalarP,
}


impl fmt::Display for JacobianPoint {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "({},{},{})", self.x, self.y, self.z)
    }
}

impl From<Point> for JacobianPoint {
    fn from(p: Point) -> Self {
        JacobianPoint{
            x:p.x,
            y:p.y,
            z:ScalarP(BigUint::one())
        }
    }
}

impl PartialEq for JacobianPoint {
    fn eq(&self, other: &JacobianPoint) -> bool {
        if self.x == other.x && self.y == other.y && self.z == other.z {
            return true;
        }

        let u1 = self.x.borrow().mul(&other.z.borrow().pow(&CONTEXT.two));
        let u2 = other.x.borrow().mul(&self.z.borrow().pow(&CONTEXT.two));

        let s1 = self.y.borrow().mul(&other.z.borrow().pow(&CONTEXT.three));
        let s2 = other.y.borrow().mul(&self.z.borrow().pow(&CONTEXT.three));

        if u1 == u2 && s1 == s2 {
            return true;
        }

        false
    }
}
impl Eq for JacobianPoint {}

impl JacobianPoint {
    pub fn double(&self) -> Option<JacobianPoint> {
        jacobian_point_double(&self)
    }
    pub fn normalize(self) -> JacobianPoint {
        JacobianPoint::from( Point::from(self))
    }
    pub fn as_bytes(self) -> [u8;33] {
        Point::from(self).as_bytes()
    }

    pub fn from_bytes(bytes : &[u8]) -> Option<Self> {
        if bytes.len()!=33 {
            return None;
        }
        Some(JacobianPoint::from( Point::from_bytes(bytes).unwrap() ))
    }

    pub fn as_uncompressed_bytes(self) -> [u8;64] {
        Point::from(self).as_uncompressed_bytes()
    }

    pub fn from_uncompressed_bytes(bytes : &[u8]) -> Option<Self> {
        if bytes.len()!=64 {
            return None;
        }
        Some(JacobianPoint::from( Point::from_uncompressed_bytes(bytes).unwrap() ))
    }

    pub fn mul(&self, n : &ScalarN) -> Self {
        jacobian_point_mul(self, n).unwrap()
    }

}

impl Add for JacobianPoint {
    type Output = JacobianPoint;

    fn add(self, other: JacobianPoint) -> JacobianPoint {
        jacobian_point_add(Some(&self), Some(&other)).unwrap()
    }
}


pub fn jacobian_point_double(p : &JacobianPoint) -> Option<JacobianPoint> {
    if p.y.0.is_zero() {
        return None;
    }
    let s = CONTEXT.four.borrow().mul(&p.x).mul( &p.y.clone()
        .pow(&CONTEXT.two) );
    let m = CONTEXT.three.borrow().mul( &p.x.borrow().pow(&CONTEXT.two));
    let x = m.borrow().pow(&CONTEXT.two).sub( &s.clone()
        .mul(&CONTEXT.two));
    let y = m.borrow().mul( &s.sub(&x) ).sub( &CONTEXT.eight.clone()
        .mul(&p.y.borrow().pow(&CONTEXT.four)));
    let z = CONTEXT.two.borrow().mul(&p.y).mul(&p.z);
    Some(JacobianPoint{x,y,z})
}

pub fn mixed_point_add(p1 : Option<&JacobianPoint>, p2 : Option<&Point>) -> Option<JacobianPoint> {
    match (p1,p2) {
        (None, None) => None,
        (Some(p1), None) => Some(p1.clone()),
        (None, Some(p2)) => Some(JacobianPoint::from( p2.clone())),
        (Some(p1), Some(p2)) => {

            let u1 = p1.x.clone();
            let u2 = p2.x.borrow().mul(&p1.z.borrow().pow(&CONTEXT.two));

            let s1 = p1.y.clone();
            let s2 = p2.y.borrow().mul(&p1.z.borrow().pow(&CONTEXT.three));

            if u1==u2 {
                if s1==s2 {
                    return jacobian_point_double(p1);
                } else {
                    return None;
                }
            }
            let h = u2.sub(&u1);
            let r = s2.sub(&s1);
            let x3 = r.pow(&CONTEXT.two)
                .sub( &h.pow(&CONTEXT.three) )
                .sub( &u1.clone().mul(&CONTEXT.two).mul(&h.pow(&CONTEXT.two) ) );

            let y3 = r.mul( &u1.mul(&h.pow(&CONTEXT.two) ).sub(&x3) )
                .sub(&s1.mul(&h.pow(&CONTEXT.three)));
            let z3 = h.mul(&p1.z);
            Some(JacobianPoint{x:x3,y:y3,z:z3})
        }
    }
}

pub fn jacobian_point_add(p1 : Option<&JacobianPoint>, p2 : Option<&JacobianPoint>) -> Option<JacobianPoint> {
    match (p1,p2) {
        (None, None) => None,
        (Some(p1), None) => Some(p1.clone()),
        (None, Some(p2)) => Some(p2.clone()),
        (Some(p1), Some(p2)) => {
            if p1.z.0.is_one() {
                return mixed_point_add(Some(&p2), Some(&Point::from(p1.to_owned())));
            }
            if p2.z.0.is_one() {
                return mixed_point_add(Some(&p1), Some(&Point::from(p2.to_owned())));
            }

            let u1 = p1.x.borrow().mul(&p2.z.borrow().pow(&CONTEXT.two));
            let u2 = p2.x.borrow().mul(&p1.z.borrow().pow(&CONTEXT.two));

            let s1 = p1.y.borrow().mul(&p2.z.borrow().pow(&CONTEXT.three));
            let s2 = p2.y.borrow().mul(&p1.z.borrow().pow(&CONTEXT.three));

            if u1==u2 {
                if s1==s2 {
                    return jacobian_point_double(p1);
                } else {
                    return None;
                }
            }
            let h = u2.sub(&u1);
            let r = s2.sub(&s1);
            let x3 = r.pow(&CONTEXT.two)
                .sub( &h.pow(&CONTEXT.three) )
                .sub( &u1.borrow().mul(&CONTEXT.two).mul(&h.pow(&CONTEXT.two) ) );

            let y3 = r.mul( &u1.mul(&h.pow(&CONTEXT.two) ).sub(&x3) )
                .sub(&s1.mul(&h.pow(&CONTEXT.three)));
            let z3 = h.mul(&p1.z).mul(&p2.z);
            Some(JacobianPoint{x:x3,y:y3,z:z3})
        }
    }
}

pub fn generator_mul(n : &ScalarN) -> Option<JacobianPoint> {
    let mut acc : Option<JacobianPoint> = None;
    for (i,byte) in n.0.to_bytes_le().iter().enumerate() {
        if byte != &0u8 {
            let index = i * 255usize + (byte - 1u8) as usize;
            let point = G_MUL_CACHE[index].to_owned();
            acc = jacobian_point_add(acc.as_ref(), Some(&point));
        }
    }
    acc
}

#[allow(non_snake_case)]
pub fn jacobian_point_mul( P: &JacobianPoint, n : &ScalarN) -> Option<JacobianPoint> {
    let mut exponent = BigUint::one()<<255;
    let mut acc : Option<JacobianPoint> = None;

    loop {
        if acc.is_some() {
            acc = acc.unwrap().double();
        }
        if !(&n.0 & &exponent).is_zero() {
            acc = jacobian_point_add(acc.as_ref(), Some(P));
        }
        exponent >>= 1usize;
        if exponent.is_zero() {
            break;
        }
    }
    acc
}


#[cfg(test)]
mod tests {
    use super::*;
    use context::CONTEXT;
    use point::point::point_add;
    use rand::prelude::*;

    #[test]
    fn test_conversion() {
        let j = JacobianPoint::from(CONTEXT.G.clone());
        let p = Point::from(j.clone());

        assert_eq!(CONTEXT.G,p);

        let g2 = point_add(Some(CONTEXT.G.clone()),Some(CONTEXT.G.clone())).unwrap();
        let g2_jac = jacobian_point_add(Some(&j), Some(&j)).unwrap();

        assert_eq!(g2.clone(), Point::from(g2_jac.clone()));

        let g3 = point_add(Some(CONTEXT.G.clone()),Some(g2.clone())).unwrap();
        let g3_jac = jacobian_point_add(Some(&j), Some(&g2_jac)).unwrap();
        assert_eq!(g3.clone(), Point::from(g3_jac));
        let three = ScalarN(BigUint::one().mul(3u32));

        let g3_jac = jacobian_point_mul(&j, &three).unwrap();
        assert_eq!(g3, Point::from(g3_jac.clone()));

        let g3_generator_mul = generator_mul(&three).unwrap();
        assert_eq!(g3_jac, g3_generator_mul);

    }

    #[test]
    fn test_generator_mul() {
        let n : ScalarN = thread_rng().gen();
        let mul_big_cache = generator_mul(&n);
        let mul_g = jacobian_point_mul(&CONTEXT.G_jacobian, &n);

        assert_eq!(mul_big_cache, mul_g);
    }

    #[test]
    fn test_serialize() {
        let x = CONTEXT.G.clone().as_uncompressed_bytes();
        let y = CONTEXT.G_jacobian.clone().as_bytes();
        let a = JacobianPoint::from_uncompressed_bytes(&x).unwrap();
        let b = Point::from_bytes(&y).unwrap();
        assert_eq!(b, Point::from(a));
    }
}