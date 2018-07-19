extern crate schnorr_edu;
extern crate data_encoding;
extern crate num_bigint;
extern crate num_traits;
extern crate num_integer;
extern crate rug;

use num_bigint::BigUint;
use schnorr_edu::scalar::ScalarN;
use std::fs::File;
use std::io::Write;
use schnorr_edu::point::jacobian_point::jacobian_point_mul;
use schnorr_edu::context::CONTEXT;
use rug::Integer;

fn main() {

    let mut buffer = File::create("res/g_mul_cache.dat").unwrap();
    let g_bytes = CONTEXT.G.as_uncompressed_bytes();
    for i in 0..32 {
        for j in 0..256usize {
            let current = Integer::from(j) << (i*8);
            //can't use generator_mul if this is the cache for it
            let point = jacobian_point_mul(
                &CONTEXT.G_jacobian,
                &ScalarN(current));
            match point {
                Some(point) => buffer.write(&point.as_uncompressed_bytes() ).unwrap(),
                None => buffer.write(&g_bytes ).unwrap(),
            };
        }
    }
}