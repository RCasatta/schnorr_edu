use std::fmt;
use num_bigint::BigUint;
use num_traits::Zero;
use num_integer::Integer;
use context::CONTEXT;
use std::ops::{Mul,Sub,Add};
use num_traits::One;
use context::AFFINES_DOUBLES_CACHE;
use scalar::ScalarN;
use scalar::ScalarP;
use point::JacobianPoint;

#[derive(Clone, PartialEq, Eq, Hash, Debug)]
pub struct Point {
    pub x: ScalarP,
    pub y: ScalarP,
}

impl fmt::Display for Point {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "({}, {})", self.x, self.y)
    }
}

impl Default for Point {
    fn default() -> Self {
        Point{x: ScalarP(BigUint::zero()), y: ScalarP(BigUint::zero())}
    }
}


impl From<JacobianPoint> for Point {
    fn from(j: JacobianPoint) -> Self {
        //(X / Z^2, Y / Z^3).
        let x = j.x.mul( &j.z.pow( &CONTEXT.two ).inv()  );
        let y = j.y.mul( &j.z.pow( &CONTEXT.three ).inv() );
        Point{x,y}
    }
}

impl Point {

    pub fn on_curve(&self) -> bool {
        let pow1 = self.y.pow(&CONTEXT.two);
        let pow2 = self.x.pow(&CONTEXT.three);
        let sub = pow1.sub(&pow2);

        sub % &CONTEXT.p == CONTEXT.seven
    }

    pub fn as_bytes(&self) -> [u8;33] {
        let mut res = [0u8;33];
        if self.y.0.is_even() {
            res[0] = 0x02;
        } else {
            res[0] = 0x03;
        }
        let bytes = self.x.to_32_bytes();
        res[1..].copy_from_slice(&bytes[..]);
        res
    }

    pub fn from_bytes(bytes : &[u8]) -> Option<Self> {
        if bytes.len()!=33 {
            return None;
        }
        let x =  ScalarP::new(BigUint::from_bytes_be(&bytes[1..]));
        let y2 = x.pow(&CONTEXT.three).add(&CONTEXT.seven);

        // in secp256k1 sqrt is equal to pow( (p-1)/4 )
        let mut y = y2.pow(&CONTEXT.p_add1_div4);
        if ( bytes[0]==0x02 && y.0.is_odd() ) || ( bytes[0]==0x03 && y.0.is_even() ) {
            y = CONTEXT.p.clone().sub(&y);
        }
        Some(Point {x,y})
    }

    pub fn from_signature_x(x : &ScalarP) -> Self {
        // we don't need to check the parity cause the schnorr sign construct impose one
        let y = x.clone().pow(&CONTEXT.three).add(&CONTEXT.seven).pow(&CONTEXT.p_add1_div4);
        Point{x : x.to_owned() ,y}
    }

    pub fn mul(self, n : &ScalarN) -> Point {
        point_mul(self, n.to_owned()).unwrap()
    }
}

impl Add for Point {
    type Output = Point;

    fn add(self, other: Point) -> <Self as Add<Point>>::Output {
        point_add(Some(self), Some(other)).unwrap()
    }
}


pub fn point_mul(mut p: Point, n : ScalarN) -> Option<Point> {
    let mut r : Option<Point> = None;
    let mut n = n.0;
    let is_g = p == CONTEXT.G;

    for i in 0..256usize {
        let (ris, rem) = n.div_rem(&CONTEXT.two.0);

        if rem.is_one() {
            r = point_add(r,Some(p.clone()));
        }
        if ris.is_zero() {
            return r;
        }
        p = match is_g {
            true  => AFFINES_DOUBLES_CACHE[i].clone(),
            false => point_add(Some(p.clone()),Some(p)).unwrap(),
        };
        n = ris;
    }
    None
}

pub fn point_add(p1 : Option<Point>, p2 : Option<Point>) -> Option<Point> {
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
                let pow = p1.y.clone().mul(&CONTEXT.two).inv();
                CONTEXT.three.clone().mul(&p1.x).mul(&p1.x).mul(&pow)
            } else {
                // lam = ((p2[1] - p1[1]) * pow(p2[0] - p1[0], p - 2, p)) % p
                let pow = p2.x.clone().sub(&p1.x).inv();
                p2.y.clone().sub(&p1.y.clone()).mul(&pow)

                //let pow = finite_sub( p2.x.clone(), &p1.x, &CONTEXT.p).modpow(&CONTEXT.p_sub2, &CONTEXT.p);
                //finite_sub( p2.y.clone(), &p1.y, &CONTEXT.p ).mul(pow).rem(&CONTEXT.p)
            };
            // x3 = (lam * lam - p1[0] - p2[0]) % p
            let x3 = lam.pow(&CONTEXT.two).sub(&p1.x).sub(&p2.x);


            //(x3, (lam * (p1[0] - x3) - p1[1]) % p)
            let sub = p1.x.clone().sub(&x3);
            let y3 = lam.mul(&sub).sub(&p1.y);

            Some(Point{x:x3,y:y3})
        }
    }
}


#[cfg(test)]
mod tests {
    use num_bigint::BigUint;
    use num_integer::Integer;
    use std::str::FromStr;
    use context::CONTEXT;
    use point::point::*;
    use data_encoding::HEXLOWER;

    #[test]
    fn text_context_mul_and_add() {
        assert_eq!("115792089237316195423570985008687907853269984665640564039457584007908834671663", format!("{}", CONTEXT.p.0));
        assert_eq!("55066263022277343669578718895168534326250603453777594175500187360389116729240", format!("{}", CONTEXT.G.x.0));
        assert_eq!("32670510020758816978083085130507043184471273380659243275938904335757337482424", format!("{}", CONTEXT.G.y.0));

        let g2 = point_add(Some(CONTEXT.G.clone()), Some(CONTEXT.G.clone())).unwrap();
        assert_eq!("89565891926547004231252920425935692360644145829622209833684329913297188986597", format!("{}", g2.x.0));
        assert_eq!("12158399299693830322967808612713398636155367887041628176798871954788371653930", format!("{}", g2.y.0));

        let g2b = point_mul(CONTEXT.G.clone(), ScalarN(BigUint::one().mul(2u32))).unwrap();
        assert_eq!(g2.x, g2b.x);
        assert_eq!(g2.y, g2b.y);

        let g3 = point_add(Some(CONTEXT.G.clone()), Some(g2.clone())).unwrap();
        assert_eq!("25583027980570883691656905877401976406448868254816295069919888960541586679410", format!("{}", g3.y.0));
        assert_eq!("112711660439710606056748659173929673102114977341539408544630613555209775888121", format!("{}", g3.x.0));

        let g3b = point_mul(CONTEXT.G.clone(), ScalarN(BigUint::one().mul(3u32))).unwrap();
        assert_eq!("112711660439710606056748659173929673102114977341539408544630613555209775888121", format!("{}", g3b.x.0));

        let g3b = point_mul(CONTEXT.G.clone(), ScalarN(BigUint::one().mul(3u32))).unwrap();
        assert_eq!(g3.x, g3b.x);
        assert_eq!(g3.y, g3b.y);

        let g8675309 = point_mul(CONTEXT.G.clone(), ScalarN(BigUint::from_str("8675309").unwrap())).unwrap();
        assert_eq!("66641067246008511739397675128206923493293851901978595085468284019495272794983", format!("{}", g8675309.x.0));
        assert_eq!("22882405661336615738255795181502754819791112635438119114432507482219105379189", format!("{}", g8675309.y.0));

        assert!(&CONTEXT.p.0.is_odd());
    }


    #[test]
    fn test_point() {
        let x_bytes = CONTEXT.G.as_bytes();
        let x = HEXLOWER.encode(&x_bytes[..]);
        assert_eq!(x, "0279be667ef9dcbbac55a06295ce870b07029bfcdb2dce28d959f2815b16f81798");

        assert!(CONTEXT.G.on_curve());

        let g_deserialized = Point::from_bytes(&x_bytes).unwrap();
        assert_eq!(&CONTEXT.G.x, &g_deserialized.x);
        assert_eq!(&CONTEXT.G.y, &g_deserialized.y);
    }
}