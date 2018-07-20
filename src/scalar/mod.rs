use crypto::sha2::Sha256;
use crypto::digest::Digest;
use rug::Integer;
pub use self::scalar_n::ScalarN;
pub use self::scalar_p::ScalarP;
use data_encoding::HEXLOWER;
use util::rug::integer_from_bytes;

pub mod scalar_n;
pub mod scalar_p;



pub fn sha256(input : &[u8]) -> Integer {
    let mut hashed = [0u8;32];
    let mut hasher = Sha256::new();
    hasher.input(input);
    hasher.result(&mut hashed);
    integer_from_bytes( &hashed[..])
}


pub fn concat_and_hash(a : &[u8], b : &[u8], c : &[u8]) -> ScalarN {
    let mut vec = Vec::with_capacity(a.len()+b.len()+c.len());
    vec.extend(a);
    vec.extend(b);
    vec.extend(c);
    ScalarN::new(sha256(&vec))
}

fn to_32_bytes(val : &Integer) -> [u8;32] {
    let mut string = val.to_string_radix(16);
    if string.len() % 2 == 1 {
        string = format!("0{}", string);
    }
    let bytes = HEXLOWER.decode( string.as_bytes() ).unwrap();
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

