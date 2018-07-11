use crypto::sha2::Sha256;
use num_bigint::BigUint;
use crypto::digest::Digest;
use std::ops::{Sub,Add,Rem};
use context::CONTEXT;


struct ScalarN(BigUint);
struct ScalarP(BigUint);

impl ScalarN {
    pub fn new(val: BigUint) -> Self {
        assert!(val < CONTEXT.n);
        ScalarN(val)
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
        ScalarN::new(finite_sub(self.0, &other.0, &CONTEXT.n))
    }
}


impl ScalarP {
    pub fn new(val: BigUint) -> Self {
        assert!(val < CONTEXT.p);
        ScalarP(val)
    }
    pub fn from_bytes(bytes: &[u8]) -> Self {
        Self::new(BigUint::from_bytes_be(bytes))
    }
    pub fn to_32_bytes(&self) -> [u8; 32] {
        to_32_bytes(&self.0)
    }
}
impl Sub for ScalarP {
    type Output = ScalarP;

    fn sub(self, other: ScalarP) -> <Self as Sub<ScalarP>>::Output {
        ScalarP::new(finite_sub(self.0, &other.0, &CONTEXT.p))
    }
}


pub fn finite_sub(a : BigUint, b : &BigUint, p_or_n : &BigUint) -> BigUint{
    if a > *b {
        a.sub(b)
    } else {
        finite_sub(a.add(p_or_n), b, p_or_n)
    }
}

pub fn sha256(input : &[u8]) -> BigUint {
    let mut hashed = [0u8;32];
    let mut hasher = Sha256::new();
    hasher.input(input);
    hasher.result(&mut hashed);
    BigUint::from_bytes_be( &hashed[..])
}

pub fn jacobi(x : &BigUint) -> BigUint {
    x.modpow(&CONTEXT.p_sub1_div2,&CONTEXT.p)
}

pub fn concat_and_hash(a : &[u8], b : &[u8], c : &[u8]) -> BigUint {
    let mut vec = Vec::with_capacity(a.len()+b.len()+c.len());
    vec.extend(a);
    vec.extend(b);
    vec.extend(c);
    sha256(&vec).rem(&CONTEXT.n)
}

pub fn to_32_bytes(val : &BigUint) -> [u8;32] {
    let bytes = val.to_bytes_be();
    let mut result = [0u8;32];
    let start = 32-bytes.len();
    assert!(start<=32);
    for i in start..32usize {
        result[i]=bytes[i-start];
    }
    result
}

pub fn vec_to_32_bytes(val : &Vec<u8>) -> [u8;32] {
    let mut result = [0u8;32];
    result.copy_from_slice(val);
    result
}
