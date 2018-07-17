use num_bigint::BigUint;
use num_bigint::BigInt;
use std::ops::{Sub,Add,Rem,Mul};
use context::CONTEXT;
use rand::distributions::Distribution;
use rand::distributions::Standard;
use rand::Rng;
use num_integer::Integer;
use super::to_32_bytes;
use super::finite_sub;
use std::fmt;
use num_traits::ToPrimitive;
use std::ops::SubAssign;
use std::ops::DivAssign;
use num_traits::Zero;

#[derive(Clone, PartialEq, Eq, Hash, Debug)]
pub struct ScalarN(pub BigUint);


impl fmt::Display for ScalarN {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{}", self.0)
    }
}

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
        ScalarN::new(finite_sub(&self.0, &other.0, &CONTEXT.n.0))
    }
}
impl Add for ScalarN {
    type Output = ScalarN;

    fn add(self, other: ScalarN) -> <Self as Add<ScalarN>>::Output {
        ScalarN::new(self.0.add(other.0) )
    }
}

impl<'a> Mul<&'a ScalarN> for ScalarN {
    type Output = ScalarN;

    fn mul(self, other: &ScalarN) -> ScalarN {
        ScalarN::new(self.0.mul(&other.0) )
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

/*
   i ← 0
   while (d > 0) do
       if (d mod 2) = 1 then
           di ← d mods 2w
           d ← d − di
       else
           di = 0
       d ← d/2
       i ← i + 1
   return (di−1, di-2, …, d0)
*/
impl ScalarN {
    pub fn to_wnaf(self, w : i8) -> Vec<i8> {
        assert!(w>1 && w<7);

        let mut d = BigInt::from(self.0);
        let mut naf = Vec::with_capacity(256usize);
        let two_raised_w = BigInt::from(2i8.pow(w as u32));
        let two_raised_w_sub1 = BigInt::from(2i8.pow( (w-1) as u32));
        while !d.is_zero() {
            if d.is_odd() {
                let mods = mods(&d, &two_raised_w, &two_raised_w_sub1);
                naf.push(mods.to_i8().unwrap());
                d.sub_assign(mods);
            } else {
                naf.push(0i8);
            }
            d.div_assign(2i32);
        }
        naf.reverse();
        naf
    }

    pub fn from_naf(v : Vec<i8>) -> Self{
        let mut acc = BigInt::zero();
        for (i,el) in v.iter().enumerate() {
            if i>0 {
                acc*=2;
            }
            acc += *el;
        }
        ScalarN( acc.to_biguint().unwrap() )
    }
}

/*
   if (d mod 2w) >= 2w−1
       return (d mod 2w) − 2w
   else
       return d mod 2w
*/

fn mods(d: &BigInt, two_raised_w : &BigInt, two_raised_w_sub1: &BigInt) -> BigInt {
    let a = d.rem(two_raised_w);
    match a >= *two_raised_w_sub1 {
        true =>  a - two_raised_w,
        false => a,
    }
}




#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_naf() {
        let n = ScalarN(BigUint::from(7u32));
        //println!("n: {:#018b}", n.0);
        //println!("{:?}", );
        let expected = [1i8,0,0,-1];
        let naf = n.clone().to_wnaf(2);
        assert_eq!(expected.to_vec(), naf);
        assert_eq!(n , ScalarN::from_naf(naf));

        for i in 1..160u32 {
            let n = ScalarN(BigUint::from(i));

            assert_eq!(n.clone() , ScalarN::from_naf(n.clone().to_wnaf(4)));
            assert_eq!(n.clone() , ScalarN::from_naf(n.clone().to_wnaf(3)));
            assert_eq!(n.clone() , ScalarN::from_naf(n.clone().to_wnaf(2)));
        }

    }
}