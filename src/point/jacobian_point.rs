use scalar::ScalarP;
use point::Point;
use num_bigint::BigUint;
use num_traits::One;
use context::CONTEXT;
use std::ops::Mul;
use std::ops::Sub;
use std::fmt;
use num_traits::Zero;

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
                    panic!("POINT_AT_INFINITY");
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
        assert_eq!(g3, Point::from(g3_jac));
    }
}