use context::CONTEXT;
use num_bigint::BigUint;
use std::fmt;
use scalar::ScalarN;
use scalar::ScalarP;

#[allow(non_snake_case)]
#[derive(Clone, PartialEq, Eq, Hash, Debug)]
pub struct Signature {
    pub Rx: ScalarP,
    pub s: ScalarN,
}

impl fmt::Display for Signature {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "(Rx {:?}, s {:?})", self.Rx, self.s)
    }
}

#[allow(non_snake_case)]
impl Signature {
    pub fn new(Rx : ScalarP, s : ScalarN) -> Self {
        Signature {Rx,s}
    }
    pub fn from_bytes(bytes : &[u8]) -> Self {
        assert_eq!(bytes.len(),64);
        let Rx = BigUint::from_bytes_be(&bytes[..32]);
        assert!( Rx < CONTEXT.p.0);
        let Rx = ScalarP(Rx);
        let s = BigUint::from_bytes_be(&bytes[32..]) ;
        assert!( s < CONTEXT.n.0);
        let s = ScalarN(s);
        Signature {Rx,s}
    }

    pub fn as_bytes(&self) -> Vec<u8> {
        let mut vec = Vec::with_capacity(64);
        vec.extend(&self.Rx.to_32_bytes() );
        vec.extend(&self.s.to_32_bytes() );
        vec
    }
}
