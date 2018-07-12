use num_bigint::BigUint;
use std::ops::{Sub,Add,Rem,Mul};
use context::CONTEXT;
use rand::distributions::Distribution;
use rand::distributions::Standard;
use rand::Rng;
use super::to_32_bytes;
use super::finite_sub;


#[derive(Clone, PartialEq, Eq, Hash, Debug)]
pub struct ScalarN(pub BigUint);


impl ScalarN {
    pub fn new(val: BigUint) -> Self {
        match val < CONTEXT.n.0 {
            true  => ScalarN(val),
            false => ScalarN(val.rem(CONTEXT.n.0.clone())),
        }
    }
    pub fn from_bytes(bytes: &[u8]) -> Self {
        Self::new(BigUint::from_bytes_be(bytes))
    }
    pub fn to_32_bytes(&self) -> [u8; 32] {
        to_32_bytes(&self.0)
    }
}
impl Sub for ScalarN {
    type Output = ScalarN;

    fn sub(self, other: ScalarN) -> <Self as Sub<ScalarN>>::Output {
        ScalarN::new(finite_sub(self.0, &other.0, &CONTEXT.n.0))
    }
}
impl Add for ScalarN {
    type Output = ScalarN;

    fn add(self, other: ScalarN) -> <Self as Add<ScalarN>>::Output {
        ScalarN::new(self.0.add(other.0) )
    }
}
impl Mul for ScalarN {
    type Output = ScalarN;

    fn mul(self, other: ScalarN) -> <Self as Mul<ScalarN>>::Output {
        ScalarN::new(self.0.mul(other.0) )
    }
}
impl Distribution<ScalarN> for Standard {
    fn sample<R: Rng + ?Sized>(&self, rng: &mut R) -> ScalarN {
        let mut bytes = [0u8;32];
        loop {
            rng.fill_bytes(&mut bytes);
            let be = BigUint::from_bytes_be(&bytes);
            if be < CONTEXT.n.0 {
                return ScalarN::new(be);
            }
        }
    }
}

