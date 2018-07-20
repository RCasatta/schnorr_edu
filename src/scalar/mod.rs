use crypto::sha2::Sha256;
use crypto::digest::Digest;
use std::ops::{Sub,Add};
use rug::Integer;
pub use self::scalar_n::ScalarN;
pub use self::scalar_p::ScalarP;
use data_encoding::HEXLOWER;
use util::rug::integer_from_bytes;

pub mod scalar_n;
pub mod scalar_p;


fn finite_sub(a : &Integer, b : &Integer, p_or_n : &Integer) -> Integer{
    if a > b {
        a.sub(b).into()
    } else {
        finite_sub(&a.add(p_or_n).into(), b, p_or_n)
    }
}

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


#[cfg(test)]
mod tests {
    use apint::ApInt;
    use rand::thread_rng;
    use rand::Rng;

    #[test]
    fn test_apint() {
        let mut rng = thread_rng();
        let val : u64 = rng.gen();
        let _apint = ApInt::from_u64(val);

        //println!("{}",apint.as_string_with_radix(Radix::new(10).unwrap()));
        //let biguint = BigUint::from(val);
        //println!("{:?}",biguint);

    }


}
