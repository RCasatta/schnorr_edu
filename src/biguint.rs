use crypto::sha2::Sha256;
use num_bigint::BigUint;
use context::Context;
use crypto::digest::Digest;
use std::ops::{Sub,Add,Rem};

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

pub fn jacobi(x : &BigUint, context : &Context) -> BigUint {
    x.modpow(&context.p_sub1_div2,&context.p)
}

pub fn concat_and_hash(a : &[u8], b : &[u8], c : &[u8], context: &Context) -> BigUint {
    let mut vec = Vec::with_capacity(a.len()+b.len()+c.len());
    vec.extend(a);
    vec.extend(b);
    vec.extend(c);
    sha256(&vec).rem(&context.n)
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