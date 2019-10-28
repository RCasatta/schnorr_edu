use super::to_32_bytes;
use context::CONTEXT;
use rand::distributions::Distribution;
use rand::distributions::Standard;
use rand::Rng;
use rug::Integer;
use std::fmt;
use std::ops::{Add, Div, Mul, Rem, Sub};
use util::rug::integer_from_bytes;

#[derive(Clone, PartialEq, Eq, Hash, Debug)]
pub struct ScalarP(pub Integer);

impl fmt::Display for ScalarP {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{}", self.0)
    }
}

impl ScalarP {
    pub fn new(val: Integer) -> Self {
        match val < CONTEXT.p.0 {
            true => ScalarP(val),
            false => ScalarP(val.rem(&CONTEXT.p.0)), // TODO not sure if panic here
        }
    }
    pub fn from_bytes(bytes: &[u8]) -> Self {
        Self::new(integer_from_bytes(bytes))
    }
    pub fn to_32_bytes(&self) -> [u8; 32] {
        to_32_bytes(&self.0)
    }
    pub fn pow(&self, n: &ScalarP) -> Self {
        /*if self.0 == BigUint::one() {
            return ScalarP(BigUint::one());
        }*/
        ScalarP(self.0.clone().pow_mod(&n.0, &CONTEXT.p.0).unwrap())
    }
    pub fn inv(&self) -> Self {
        ScalarP(
            self.0
                .clone()
                .pow_mod(&CONTEXT.p_sub2.0, &CONTEXT.p.0)
                .unwrap(),
        )
    }

    pub fn is_square(&self) -> bool {
        self.pow(&CONTEXT.p_sub1_div2).0 == 1
    }

}
impl<'a> Sub<&'a ScalarP> for ScalarP {
    type Output = ScalarP;

    fn sub(self, other: &ScalarP) -> ScalarP {
        let value = if self.0 > other.0 {
            self.0.sub(&other.0)
        } else {
            self.0.add(&CONTEXT.p.0).sub(&other.0)
        };

        ScalarP::new(value)
    }
}

impl<'a> Add<&'a ScalarP> for ScalarP {
    type Output = ScalarP;

    fn add(self, other: &ScalarP) -> ScalarP {
        ScalarP::new(self.0.add(&other.0))
    }
}
impl<'a> Mul<&'a ScalarP> for ScalarP {
    type Output = ScalarP;

    fn mul(self, other: &ScalarP) -> ScalarP {
        ScalarP::new(self.0.mul(&other.0))
    }
}

/*
impl<'a, 'b> Mul<&'b ScalarP> for &'a ScalarP {
    type Output = ScalarP;

    fn mul(self, other: &'b ScalarP) -> ScalarP {
        ScalarP::new((&other.0 * &self.0).into() )
    }
}
*/

//TODO add mul u32

impl<'a> Rem<&'a ScalarP> for ScalarP {
    type Output = ScalarP;

    fn rem(self, other: &ScalarP) -> ScalarP {
        ScalarP(self.0.rem(&other.0))
    }
}
impl<'a> Div<&'a ScalarP> for ScalarP {
    type Output = ScalarP;

    fn div(self, other: &ScalarP) -> ScalarP {
        ScalarP::new(self.0.div(&other.0))
    }
}

impl Distribution<ScalarP> for Standard {
    fn sample<R: Rng + ?Sized>(&self, rng: &mut R) -> ScalarP {
        let mut bytes = [0u8; 32];
        loop {
            rng.fill_bytes(&mut bytes);
            let be = integer_from_bytes(&bytes);
            if be < CONTEXT.p.0 {
                return ScalarP::new(be);
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use context::CONTEXT;

    #[test]
    fn test_inv() {
        assert!(CONTEXT.G.x.clone().inv().mul(&CONTEXT.G.x).0 == 1);
    }

    #[test]
    fn test_borrow() {
        let a = Integer::from(1);
        let b = Integer::from(1);
        let c: Integer = (&a + &b).into();

        assert_eq!(c, Integer::from(2u32));
    }
}
