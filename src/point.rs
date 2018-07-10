use std::fmt;
use num_bigint::BigUint;
use num_traits::Zero;
use num_integer::Integer;
use context::Context;
use biguint::{finite_sub, to_32_bytes};
use std::ops::{Mul,Sub,Rem,Add};
use num_traits::One;

#[derive(Clone, PartialEq, Eq, Hash)]
pub struct Point {
    pub x: BigUint,
    pub y: BigUint,
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
        let sub = finite_sub(pow1, &pow2, &context.p);

        sub % &context.p == context.seven
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

    pub fn from_signature_x(x : &BigUint, context : &Context) -> Self {
        // we don't need to check the parity cause the schnorr sign construct impose one
        let y = x.modpow(&context.three, &context.p).add(&context.seven).modpow(&context.p_add1_div4, &context.p);
        Point{x : (*x).clone() ,y}
    }

    pub fn mul(self, n : BigUint, context : &Context) -> Option<Point> {
        point_mul(Some(self), n, context)
    }

    pub fn add(self, p2 : &Option<Point>, context : &Context) -> Option<Point> {  //TODO before using standard trait Add, we must use lazy_static for Context?
        point_add(&Some(self), p2, context)
    }
}


pub fn point_mul(mut p: Option<Point>, mut n : BigUint, context : &Context) -> Option<Point> {
    let mut r : Option<Point> = None;

    loop {
        let (ris, rem) = n.div_rem(&context.two);

        if rem.is_one() {
            r = point_add(&r,&p, context);
        }
        if ris.is_zero() {
            return r;
        }
        p = point_add(&p,&p, context);
        n = ris;

    }
}

pub fn point_add(p1 : &Option<Point>, p2 : &Option<Point>, context : &Context) -> Option<Point> {  // TODO change to Option<&Point> !!!
    match (p1,p2) {
        (None, None) => None,
        (Some(p1), None) => Some(p1.clone()),
        (None, Some(p2)) => Some(p2.clone()),
        (Some(p1), Some(p2)) => {
            if  p1.x == p2.x && p1.y != p2.y {
                return None;
            }
            let lam = if  p1 == p2 {
                let option = context.map.get(p1);
                if option.is_some() {
                    return Some((*option.unwrap()).clone());
                }
                // lam = (3 * p1[0] * p1[0] * pow(2 * p1[1], p - 2, p)) % p
                let pow = p1.y.clone().mul(2u32).modpow(&context.p_sub2, &context.p);
                context.three.clone().mul(&p1.x).rem(&context.p).mul(&p1.x).rem(&context.p).mul(&pow).rem(&context.p)
            } else {
                // lam = ((p2[1] - p1[1]) * pow(p2[0] - p1[0], p - 2, p)) % p
                let pow = finite_sub( p2.x.clone(), &p1.x, &context.p).modpow(&context.p_sub2, &context.p);
                finite_sub( p2.y.clone(), &p1.y, &context.p ).mul(pow).rem(&context.p)
            };
            // x3 = (lam * lam - p1[0] - p2[0]) % p
            let x3 = lam.modpow(&context.two, &context.p);
            let x3 = finite_sub(x3,&p1.x,&context.p);
            let x3 = finite_sub(x3,&p2.x,&context.p);

            //(x3, (lam * (p1[0] - x3) - p1[1]) % p)
            let sub = finite_sub(p1.x.clone(), &x3, &context.p);
            let y3 = finite_sub(lam * sub, &p1.y, &context.p).rem(&context.p);

            Some(Point{x:x3,y:y3})
        }
    }
}


#[cfg(test)]
mod tests {
    use num_bigint::BigUint;
    use num_integer::Integer;
    use std::str::FromStr;
    use context::Context;
    use point::*;
    use data_encoding::HEXLOWER;

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
        assert_eq!(g2.x, g2b.x);
        assert_eq!(g2.y, g2b.y);

        let g3b = point_mul(Some(context.G.clone()), context.three.clone(), &context).unwrap();
        assert_eq!(g3.x, g3b.x);
        assert_eq!(g3.y, g3b.y);

        let g8675309 = point_mul(Some(context.G.clone()), BigUint::from_str("8675309").unwrap(), &context).unwrap();
        assert_eq!("66641067246008511739397675128206923493293851901978595085468284019495272794983", format!("{}", g8675309.x));
        assert_eq!("22882405661336615738255795181502754819791112635438119114432507482219105379189", format!("{}", g8675309.y));

        assert!(&context.p.is_odd());
    }


    #[test]
    fn test_point() {
        let context = Context::default();
        let x_bytes = context.G.as_bytes();
        let x = HEXLOWER.encode(&x_bytes[..]);
        assert_eq!(x, "0279be667ef9dcbbac55a06295ce870b07029bfcdb2dce28d959f2815b16f81798");

        assert!(context.G.on_curve(&context));

        let g_deserialized = Point::from_bytes(&x_bytes, &context).unwrap();
        assert_eq!(&context.G.x, &g_deserialized.x);
        assert_eq!(&context.G.y, &g_deserialized.y);
    }
}