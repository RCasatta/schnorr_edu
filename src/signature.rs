use std::ops::Add;
use context::Context;
use num_bigint::BigUint;
use biguint::to_32_bytes;
use point::Point;
use std::ops::Rem;
use std::fmt;

#[allow(non_snake_case)]
#[derive(Clone, PartialEq, Eq, Hash)]
struct Signature {
    Rx: BigUint,
    s: BigUint,
}

impl fmt::Display for Signature {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "(Rx {}, s {})", self.Rx, self.s)
    }
}

#[allow(non_snake_case)]
impl Signature {
    pub fn new(bytes : &[u8]) -> Signature {
        assert_eq!(bytes.len(),64);
        Signature {
            Rx : BigUint::from_bytes_be(&bytes[..32]),
            s :  BigUint::from_bytes_be(&bytes[32..]),
        }
    }

    pub fn as_bytes(&self) -> Vec<u8> {
        let mut vec = Vec::with_capacity(64);
        vec.extend(&to_32_bytes(&self.Rx) );
        vec.extend(&to_32_bytes(&self.s) );
        vec
    }
}
