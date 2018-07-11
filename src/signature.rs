use context::CONTEXT;
use num_bigint::BigUint;
use scalar::to_32_bytes;
use std::fmt;

#[allow(non_snake_case)]
#[derive(Clone, PartialEq, Eq, Hash)]
pub struct Signature {
    pub Rx: BigUint,
    pub s: BigUint,
}

impl fmt::Display for Signature {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "(Rx {}, s {})", self.Rx, self.s)
    }
}

#[allow(non_snake_case)]
impl Signature {
    pub fn new(Rx : BigUint, s : BigUint) -> Self {
        Signature {Rx,s}
    }
    pub fn from_bytes(bytes : &[u8]) -> Self {
        assert_eq!(bytes.len(),64);
        let Rx = BigUint::from_bytes_be(&bytes[..32]);
        assert!( Rx < CONTEXT.p);
        let s = BigUint::from_bytes_be(&bytes[32..]);
        assert!( s < CONTEXT.n);
        Signature {Rx,s}
    }

    pub fn as_bytes(&self) -> Vec<u8> {
        let mut vec = Vec::with_capacity(64);
        vec.extend(&to_32_bytes(&self.Rx) );
        vec.extend(&to_32_bytes(&self.s) );
        vec
    }
}
