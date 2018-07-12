extern crate schnorr_edu;
extern crate data_encoding;

use schnorr_edu::context::CONTEXT;
use schnorr_edu::point::point::point_add;
use data_encoding::HEXLOWER;

fn main() {

    let mut current = CONTEXT.G.clone();
    println!("{}", HEXLOWER.encode( &current.as_bytes()) );
    for _ in 0..256 {
        let double = point_add(Some(current.clone()), Some(current.clone())).unwrap();
        println!("{}", HEXLOWER.encode(&double.as_bytes()));
        current = double;
    }
}