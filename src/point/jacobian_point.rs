use scalar::ScalarP;
use scalar::ScalarN;
use point::Point;
use num_bigint::BigUint;
use num_traits::One;
use context::CONTEXT;
use context::JACOBIAN_DOUBLES_CACHE;
use std::ops::{Mul,Sub,Add};
use std::fmt;
use num_traits::Zero;
use num_integer::Integer;

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

        let u1 = self.x.clone().mul(&other.z.clone().pow(&CONTEXT.two));
        let u2 = other.x.clone().mul(&self.z.clone().pow(&CONTEXT.two));

        let s1 = self.y.clone().mul(&other.z.clone().pow(&CONTEXT.three));
        let s2 = other.y.clone().mul(&self.z.clone().pow(&CONTEXT.three));

        if u1 == u2 && s1 == s2 {
            return true;
        }

        false
    }
}
impl Eq for JacobianPoint {}

impl JacobianPoint {
    pub fn double(self) -> JacobianPoint {
        jacobian_point_double(self)
    }
    pub fn normalize(self) -> JacobianPoint {
        JacobianPoint::from( Point::from(self))
    }
    pub fn as_bytes(&self) -> [u8;64] {
        let mut res = [0u8;64];
        let mut p = self.to_owned();
        if !p.z.0.is_one() {
            p = p.normalize();
        }
        res[..32].copy_from_slice( &p.x.to_32_bytes() );
        res[32..].copy_from_slice( &p.y.to_32_bytes() );
        res
    }

    pub fn from_bytes(bytes : &[u8]) -> Option<Self> {
        if bytes.len()!=64 {
            return None;
        }
        let x = BigUint::from_bytes_be(&bytes[..32]);
        let y = BigUint::from_bytes_be(&bytes[..32]);
        if x > CONTEXT.p.0 || y > CONTEXT.p.0 {
            return None;
        }
        Some(
            JacobianPoint{
                x:ScalarP(x),
                y:ScalarP(y),
                z:ScalarP(BigUint::one()),
            })
    }
    pub fn mul(self, n : &ScalarN) -> Self {
        jacobian_point_mul(self, n.to_owned()).unwrap()
    }
}

impl Add for JacobianPoint {
    type Output = JacobianPoint;

    fn add(self, other: JacobianPoint) -> JacobianPoint {
        jacobian_point_add(Some(self), Some(other)).unwrap()
    }
}


pub fn jacobian_point_double(p : JacobianPoint) -> JacobianPoint {
    if p.y.0.is_zero() {
        println!("POINT_AT_INFINITY");
    }
    let s = CONTEXT.four.clone().mul(&p.x).mul( &p.y.clone().pow(&CONTEXT.two) );
    let m = CONTEXT.three.clone().mul( &p.x.clone().pow(&CONTEXT.two));
    let x = m.clone().pow(&CONTEXT.two).sub( &s.clone().mul(&CONTEXT.two));
    let y = m.clone().mul( &s.sub(&x) ).sub( &CONTEXT.eight.clone().mul(&p.y.clone().pow(&CONTEXT.four)));
    let z = CONTEXT.two.clone().mul(&p.y).mul(&p.z);
    JacobianPoint{x,y,z}
}

pub fn jacobian_point_add(p1 : Option<JacobianPoint>, p2 : Option<JacobianPoint>) -> Option<JacobianPoint> {
    match (p1,p2) {
        (None, None) => None,
        (Some(p1), None) => Some(p1.clone()),
        (None, Some(p2)) => Some(p2.clone()),
        (Some(p1), Some(p2)) => {
            let u1 = p1.x.clone().mul(&p2.z.clone().pow(&CONTEXT.two));
            let u2 = p2.x.clone().mul(&p1.z.clone().pow(&CONTEXT.two));

            let s1 = p1.y.clone().mul(&p2.z.clone().pow(&CONTEXT.three));
            let s2 = p2.y.clone().mul(&p1.z.clone().pow(&CONTEXT.three));

            if u1==u2 {
                if s1==s2 {
                    return Some(jacobian_point_double(p1));
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
            let z3 = h.mul(&p1.z).mul(&p2.z);
            Some(JacobianPoint{x:x3,y:y3,z:z3})
        }
    }
}


pub fn jacobian_point_mul(mut p: JacobianPoint, n : ScalarN) -> Option<JacobianPoint> {
    let mut r : Option<JacobianPoint> = None;
    let mut n = n.0;
    let is_g = p == CONTEXT.G_jacobian;

    for i in 0..256usize {
        let (ris, rem) = n.div_rem(&CONTEXT.two.0);

        if rem.is_one() {
            r = jacobian_point_add(r,Some(p.clone()));
        }
        if ris.is_zero() {
            return r;
        }
        p = match is_g {
            true  => JACOBIAN_DOUBLES_CACHE[i].clone(),
            false => jacobian_point_add(Some(p.clone()),Some(p)).unwrap(),
        };
        n = ris;
    }
    None
}


#[cfg(test)]
mod tests {
    use super::*;
    use context::CONTEXT;
    use point::point::point_add;

    #[test]
    fn test_conversion() {
        let j = JacobianPoint::from(CONTEXT.G.clone());
        let p = Point::from(j.clone());

        assert_eq!(CONTEXT.G,p);

        let g2 = point_add(Some(CONTEXT.G.clone()),Some(CONTEXT.G.clone())).unwrap();
        let g2_jac = jacobian_point_add(Some(j.clone()), Some(j.clone())).unwrap();

        assert_eq!(g2.clone(), Point::from(g2_jac.clone()));

        let g3 = point_add(Some(CONTEXT.G.clone()),Some(g2.clone())).unwrap();
        let g3_jac = jacobian_point_add(Some(j.clone()), Some(g2_jac.clone())).unwrap();
        assert_eq!(g3.clone(), Point::from(g3_jac));

        let g3_jac = jacobian_point_mul(j.clone(), ScalarN(BigUint::one().mul(3u32) )).unwrap();
        assert_eq!(g3, Point::from(g3_jac));
    }
}