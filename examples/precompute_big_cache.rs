extern crate schnorr_edu;
extern crate data_encoding;
extern crate num_bigint;
extern crate num_traits;
extern crate num_integer;

use schnorr_edu::point::point::point_add;
use schnorr_edu::point::generator_mul;
use num_bigint::BigUint;
use schnorr_edu::scalar::ScalarN;
use num_traits::One;
use std::fs::File;
use std::io::Write;


fn main() {

    let mut buffer = File::create("big_cache.dat").unwrap();
    for i in 0..32 {
        for j in 1..256usize {
            let current = BigUint::from(j) << (i*8);
            let point = generator_mul(&ScalarN(current.clone())).unwrap();
            buffer.write(&point.as_bytes() ).unwrap();
        }
    }
}