use context::CONTEXT;
use scalar::ScalarN;
use scalar::ScalarP;
use std::fmt;
use util::rug::integer_from_bytes;

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
    pub fn new(Rx: ScalarP, s: ScalarN) -> Self {
        Signature { Rx, s }
    }
    pub fn from_bytes(bytes: &[u8]) -> Result<Self,()> {
        assert_eq!(bytes.len(), 64);
        let Rx = integer_from_bytes(&bytes[..32]);
        if Rx >= CONTEXT.p.0 {
            return Err(());
        }
        let Rx = ScalarP(Rx);
        let s = integer_from_bytes(&bytes[32..]);
        if s >= CONTEXT.n.0 {
            return Err(());
        }
        let s = ScalarN(s);
        Ok(Signature { Rx, s })
    }

    pub fn as_bytes(&self) -> Vec<u8> {
        let mut vec = Vec::with_capacity(64);
        vec.extend(&self.Rx.to_32_bytes());
        vec.extend(&self.s.to_32_bytes());
        vec
    }
}
