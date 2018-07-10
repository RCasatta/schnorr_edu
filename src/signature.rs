use context::Context;
use num_bigint::BigUint;
use biguint::to_32_bytes;
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
    pub fn new(bytes : &[u8], context: &Context) -> Signature {
        assert_eq!(bytes.len(),64);
        let Rx = BigUint::from_bytes_be(&bytes[..32]);
        assert!( Rx < context.p);
        let s = BigUint::from_bytes_be(&bytes[32..]);
        assert!( s < context.n);
        Signature {Rx,s}
    }

    pub fn as_bytes(&self) -> Vec<u8> {
        let mut vec = Vec::with_capacity(64);
        vec.extend(&to_32_bytes(&self.Rx) );
        vec.extend(&to_32_bytes(&self.s) );
        vec
    }
}
