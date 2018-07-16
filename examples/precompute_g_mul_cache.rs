extern crate schnorr_edu;
extern crate data_encoding;
extern crate num_bigint;
extern crate num_traits;
extern crate num_integer;

use num_bigint::BigUint;
use schnorr_edu::scalar::ScalarN;
use std::fs::File;
use std::io::Write;
use schnorr_edu::point::jacobian_point::jacobian_point_mul;
use schnorr_edu::context::CONTEXT;

fn main() {

    let mut buffer = File::create("res/g_mul_cache.dat").unwrap();
    for i in 0..32 {
        for j in 1..256usize {
            let current = BigUint::from(j) << (i*8);
            //can't use generator_mul if this is the cache for it
            let point = jacobian_point_mul(
                &CONTEXT.G_jacobian,
                &ScalarN(current))
                .unwrap();
            buffer.write(&point.as_uncompressed_bytes() ).unwrap();
        }
    }
}