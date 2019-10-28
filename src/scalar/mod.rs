pub use self::scalar_n::ScalarN;
pub use self::scalar_p::ScalarP;
use crypto::digest::Digest;
use crypto::sha2::Sha256;
use data_encoding::HEXLOWER;
use rug::Integer;
use util::rug::integer_from_bytes;

pub mod scalar_n;
pub mod scalar_p;

#[allow(non_snake_case)]
pub fn sha256_BIPSchnorr(input: &[u8]) -> Integer {
    tagged_sha256(b"BIPSchnorr", input)
}

#[allow(non_snake_case)]
pub fn sha256_BIPSchnorrDerive(input: &[u8]) -> Integer {
    tagged_sha256(b"BIPSchnorrDerive", input)
}

pub fn tagged_sha256(tag :&[u8], input: &[u8]) -> Integer {
    let mut hashed = [0u8; 32];
    let mut hasher = Sha256::new();
    let tag_hash = sha256(tag);
    //println!("tag_hash {}", &HEXLOWER.encode(&tag_hash));
    hasher.input(&tag_hash);
    hasher.input(&tag_hash);
    hasher.input(input);
    hasher.result(&mut hashed);
    integer_from_bytes(&hashed[..])
}

pub fn sha256(input: &[u8]) -> [u8; 32] {
    let mut hashed = [0u8; 32];
    let mut hasher = Sha256::new();
    hasher.input(input);
    hasher.result(&mut hashed);
    hashed
}
pub fn sha256_int(input: &[u8]) -> Integer {
    integer_from_bytes(&sha256(input))
}

#[allow(non_snake_case)]
pub fn concat_and_hash_BIPSchnorr(a: &[u8], b: &[u8], c: &[u8]) -> ScalarN {
    let mut vec = Vec::with_capacity(a.len() + b.len() + c.len());
    vec.extend(a);
    vec.extend(b);
    vec.extend(c);
    ScalarN::new(sha256_BIPSchnorr(&vec))
}

#[allow(non_snake_case)]
pub fn concat_and_hash_BIPSchnorrDerive(a: &[u8], b: &[u8], c: &[u8]) -> ScalarN {
    let mut vec = Vec::with_capacity(a.len() + b.len() + c.len());
    vec.extend(a);
    vec.extend(b);
    vec.extend(c);
    ScalarN::new(sha256_BIPSchnorrDerive(&vec))
}


pub fn concat_and_hash(a: &[u8], b: &[u8], c: &[u8]) -> ScalarN {
    let mut vec = Vec::with_capacity(a.len() + b.len() + c.len());
    vec.extend(a);
    vec.extend(b);
    vec.extend(c);
    ScalarN::new(sha256_int(&vec))
}


fn to_32_bytes(val: &Integer) -> [u8; 32] {
    let mut string = val.to_string_radix(16);
    if string.len() % 2 == 1 {
        string = format!("0{}", string);
    }
    let bytes = HEXLOWER.decode(string.as_bytes()).unwrap();
    let mut result = [0u8; 32];
    let start = 32 - bytes.len();
    assert!(start <= 32);
    for i in start..32usize {
        result[i] = bytes[i - start];
    }
    result
}

pub fn vec_to_32_bytes(val: &Vec<u8>) -> [u8; 32] {
    let mut result = [0u8; 32];
    result.copy_from_slice(val);
    result
}
