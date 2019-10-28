use context::CONTEXT;
use context::G_MUL_CACHE;
use num_bigint::BigUint;
use num_traits::Num;
use point::Point;
use rug::Assign;
use rug::Integer;
use scalar::ScalarN;
use scalar::ScalarP;
use std::borrow::Borrow;
use std::fmt;
use std::ops::MulAssign;
use std::ops::{Add, Mul, Sub};

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
        JacobianPoint {
            x: p.x,
            y: p.y,
            z: ScalarP(Integer::from(1)),
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
    pub fn double(&self) -> Option<JacobianPoint> {
        jacobian_point_double(&self)
    }
    pub fn normalize(self) -> JacobianPoint {
        JacobianPoint::from(Point::from(self))
    }
    pub fn as_bytes(self) -> [u8; 32] {
        Point::from(self).as_bytes()
    }

    pub fn from_bytes(bytes: &[u8]) -> Option<Self> {
        if bytes.len() != 33 {
            return None;
        }
        Some(JacobianPoint::from(Point::from_bytes(bytes).unwrap()))
    }

    pub fn as_uncompressed_bytes(self) -> [u8; 64] {
        Point::from(self).as_uncompressed_bytes()
    }

    pub fn from_uncompressed_bytes(bytes: &[u8]) -> Option<Self> {
        if bytes.len() != 64 {
            return None;
        }
        Some(JacobianPoint::from(
            Point::from_uncompressed_bytes(bytes).unwrap(),
        ))
    }

    pub fn mul(&self, n: &ScalarN) -> Self {
        jacobian_point_mul_wnaf(self, n, 5i8).unwrap()
    }

    pub fn negate(self) -> Self {
        JacobianPoint {
            x: self.x,
            y: CONTEXT.p.clone() - &self.y,
            z: self.z,
        }
    }

    pub fn is_square(&self) -> bool {
        //TODO not efficient
        let p: Point = self.clone().into();
        p.y.is_square()
    }
}

impl Add for JacobianPoint {
    type Output = JacobianPoint;

    fn add(self, other: JacobianPoint) -> JacobianPoint {
        jacobian_point_add(Some(&self), Some(&other)).unwrap()
    }
}

pub fn jacobian_point_double(p: &JacobianPoint) -> Option<JacobianPoint> {
    if p.y.0 == 0 {
        return None;
    }
    let p_x_pow2 = p.x.clone().mul(&p.x);
    let p_y_pow2 = p.y.clone().mul(&p.y);
    let p_y_pow4 = p_y_pow2.clone().mul(&p_y_pow2);
    let s = p_y_pow2.mul(&p.x).mul(&CONTEXT.four);
    let m = p_x_pow2.mul(&CONTEXT.three);
    let x = m.clone().mul(&m).sub(&s.clone().mul(&CONTEXT.two));
    let y = m
        .mul(s.sub(&x).borrow())
        .sub(p_y_pow4.mul(&CONTEXT.eight).borrow());
    let z = CONTEXT.two.clone().mul(&p.y).mul(&p.z);
    Some(JacobianPoint { x, y, z })
}

pub fn mixed_point_add(p1: Option<&JacobianPoint>, p2: Option<&Point>) -> Option<JacobianPoint> {
    match (p1, p2) {
        (None, None) => None,
        (Some(p1), None) => Some(p1.clone()),
        (None, Some(p2)) => Some(JacobianPoint::from(p2.clone())),
        (Some(p1), Some(p2)) => {
            let mut p1_z_pow2 = Integer::with_capacity(512);
            p1_z_pow2.assign(&p1.z.0);
            p1_z_pow2.mul_assign(&p1.z.0);
            let p1_z_pow2 = ScalarP::new(p1_z_pow2);

            let u1 = &p1.x;

            let mut u2 = Integer::with_capacity(512);
            u2.assign(&p2.x.0);
            u2.mul_assign(&p1_z_pow2.0);
            let u2 = ScalarP::new(u2);

            let s1 = &p1.y;
            let s2 = p1_z_pow2.mul(&p1.z).mul(&p2.y);

            if *u1 == u2 {
                if *s1 == s2 {
                    return jacobian_point_double(p1);
                } else {
                    return None;
                }
            }
            let h = u2.sub(&u1);
            let h_pow2 = h.clone().mul(&h);
            //println!("capacity {}", h_pow2.0.capacity());

            let h_pow3 = h_pow2.clone().mul(&h);
            let r = s2.sub(&s1);
            let r_pow2 = r.clone().mul(&r);

            let x3 = r_pow2
                .sub(&h_pow3)
                .sub(&u1.clone().mul(&CONTEXT.two).mul(&h_pow2));

            let u1_mul_h_pow2 = h_pow2.mul(&u1);

            let y3 = r
                .mul(u1_mul_h_pow2.sub(&x3).borrow())
                .sub(h_pow3.mul(&s1).borrow());

            let z3 = h.mul(&p1.z);
            Some(JacobianPoint {
                x: x3,
                y: y3,
                z: z3,
            })
        }
    }
}

pub fn jacobian_point_add(
    p1: Option<&JacobianPoint>,
    p2: Option<&JacobianPoint>,
) -> Option<JacobianPoint> {
    match (p1, p2) {
        (None, None) => None,
        (Some(p1), None) => Some(p1.clone()),
        (None, Some(p2)) => Some(p2.clone()),
        (Some(p1), Some(p2)) => {
            let p2_z_pow2 = p2.z.clone().mul(&p2.z);
            let p1_z_pow2 = p1.z.clone().mul(&p1.z);

            let u1 = p1.x.clone().mul(&p2_z_pow2);
            let u2 = p2.x.clone().mul(&p1_z_pow2);

            let s1 = p2_z_pow2.mul(&p2.z).mul(&p1.y);
            let s2 = p1_z_pow2.mul(&p2.y).mul(&p1.z);

            if u1 == u2 {
                if s1 == s2 {
                    return jacobian_point_double(p1);
                } else {
                    return None;
                }
            }
            let h = u2.sub(&u1);
            let h_pow2 = h.clone().mul(&h);
            let h_pow3 = h_pow2.clone().mul(&h);
            let r = s2.sub(&s1);
            let r_pow2 = r.clone().mul(&r);
            let x3 = r_pow2
                .sub(&h_pow3)
                .sub(&u1.clone().mul(&CONTEXT.two).mul(&h_pow2));
            let y3 = r
                .mul(u1.mul(&h_pow2).sub(&x3).borrow())
                .sub(s1.mul(&h_pow3).borrow());
            let z3 = h.mul(&p1.z).mul(&p2.z);
            Some(JacobianPoint {
                x: x3,
                y: y3,
                z: z3,
            })
        }
    }
}

pub fn generator_mul(n: &ScalarN) -> Option<JacobianPoint> {
    let mut acc: Option<JacobianPoint> = None;
    let mut _junk: Option<JacobianPoint> = None;
    let string_radix = n.0.to_string_radix(16);
    let bi = BigUint::from_str_radix(&string_radix, 16).unwrap();
    for (i, byte) in bi.to_bytes_le().iter().enumerate() {
        let index = i * 256usize + usize::from(*byte);
        let point = G_MUL_CACHE.get(index);
        if *byte != 0u8 {
            acc = mixed_point_add(acc.as_ref(), point);
        } else {
            // the purpose of this arm is to try to achieve constant time
            // who knows if the compiler removes it, however you should not read this
            // this lib is totally unsecure
            _junk = mixed_point_add(acc.as_ref(), Some(&CONTEXT.G));
        }
    }
    acc
}

// https://en.wikipedia.org/wiki/Elliptic_curve_point_multiplication

#[allow(non_snake_case)]
pub fn jacobian_point_mul(P: &JacobianPoint, n: &ScalarN) -> Option<JacobianPoint> {
    let mut exponent: Integer = Integer::from(1) << 255;
    let mut acc: Option<JacobianPoint> = None;

    loop {
        if acc.is_some() {
            acc = acc.unwrap().double();
        }
        let val: Integer = (&n.0 & &exponent).into();
        if val != 0 {
            acc = jacobian_point_add(acc.as_ref(), Some(P));
        }
        exponent = exponent >> 1;
        if exponent == 0 {
            break;
        }
    }
    acc
}

#[allow(non_snake_case)]
pub fn jacobian_point_mul_wnaf(P: &JacobianPoint, n: &ScalarN, w: i8) -> Option<JacobianPoint> {
    assert!(w > 1 && w < 8);
    let vec = n.to_owned().to_wnaf(w);
    let times = 2i8.pow(w as u32 - 2);
    let mut positives = Vec::new();
    let mut prec = P.to_owned();
    let two_P = P.double();
    positives.push(prec.clone());
    for _ in 1..times {
        prec = jacobian_point_add(two_P.as_ref(), Some(&prec)).unwrap();
        positives.push(prec.clone());
    }
    let mut precomputed = Vec::new();

    for el in positives.iter().rev() {
        precomputed.push(el.to_owned().negate());
    }
    for el in positives.iter() {
        precomputed.push(el.to_owned());
    }
    let max = 2i8.pow(w as u32 - 1) - 1;

    let mut acc: Option<JacobianPoint> = None;

    for el in vec.iter() {
        if acc.is_some() {
            acc = acc.unwrap().double();
        }

        if *el != 0i8 {
            let index = (el + max) / 2;
            acc = jacobian_point_add(acc.as_ref(), precomputed.get(index as usize));
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

        assert_eq!(CONTEXT.G, p);

        let g2 = point_add(Some(CONTEXT.G.clone()), Some(CONTEXT.G.clone())).unwrap();
        let g2_jac = jacobian_point_add(Some(&j), Some(&j)).unwrap();

        assert_eq!(g2.clone(), Point::from(g2_jac.clone()));

        let g3 = point_add(Some(CONTEXT.G.clone()), Some(g2.clone())).unwrap();
        let g3_jac = jacobian_point_add(Some(&j), Some(&g2_jac)).unwrap();
        assert_eq!(g3.clone(), Point::from(g3_jac));
        let three = ScalarN(Integer::from(3));

        let g3_jac = jacobian_point_mul(&j, &three).unwrap();
        assert_eq!(g3, Point::from(g3_jac.clone()));

        let g3_generator_mul = generator_mul(&three).unwrap();
        assert_eq!(g3_jac, g3_generator_mul);
    }

    #[test]
    fn test_generator_mul() {
        let n: ScalarN = thread_rng().gen();
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

    #[test]
    fn test_mul_wnaf() {
        //jacobian_point_mul_4naf
        let two = ScalarN(Integer::from(2));

        let option = jacobian_point_mul_wnaf(&CONTEXT.G_jacobian, &two, 4i8);
        assert_eq!(CONTEXT.G_jacobian.clone().double(), option);

        let n: ScalarN = thread_rng().gen();

        let option = jacobian_point_mul_wnaf(&CONTEXT.G_jacobian, &n, 2i8);
        assert_eq!(generator_mul(&n), option);

        let option = jacobian_point_mul_wnaf(&CONTEXT.G_jacobian, &n, 3i8);
        assert_eq!(generator_mul(&n), option);

        let option = jacobian_point_mul_wnaf(&CONTEXT.G_jacobian, &n, 4i8);
        assert_eq!(generator_mul(&n), option);

        let option = jacobian_point_mul_wnaf(&CONTEXT.G_jacobian, &n, 5i8);
        assert_eq!(generator_mul(&n), option);

        let option = jacobian_point_mul_wnaf(&CONTEXT.G_jacobian, &n, 6i8);
        assert_eq!(generator_mul(&n), option);

        let option = jacobian_point_mul_wnaf(&CONTEXT.G_jacobian, &n, 7i8);
        assert_eq!(generator_mul(&n), option);
    }
}
