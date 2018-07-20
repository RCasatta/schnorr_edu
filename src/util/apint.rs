use apint::ApInt;
use apint::Width;

fn apint_to_bytes_le(mut apint: ApInt) -> Vec<u8> {
    let mut bytes = Vec::with_capacity(apint.width().to_usize()/8 );
    loop {
        if apint.is_zero() {
            break;
        }
        bytes.push(apint.resize_to_u8());
        apint = apint.into_checked_ashr(8).unwrap();
    }
    bytes
}



#[cfg(test)]
mod tests {
    use apint::ApInt;
    use num_bigint::BigUint;
    use rand::thread_rng;
    use rand::Rng;
    use num_traits::FromPrimitive;
    use super::*;

    #[test]
    fn test_apint() {

        let apint = ApInt::from_u64(1000);
        let biguint = BigUint::from_u32(1000).unwrap();

        assert_eq!(biguint.to_bytes_le(), apint_to_bytes_le(apint) );

        //println!("flags: {:#018b}", 260);

        //println!("{:?}", apint);
        //println!("{}", apint.resize_to_u8());

        //let apint = apint.into_checked_ashr(8).unwrap();

        //println!("{:?}", apint);
        //println!("{}", apint.resize_to_u8());

        //println!("{}",apint.as_string_with_radix(Radix::new(10).unwrap()));
        //let biguint = BigUint::from(val);
        //println!("{:?}",biguint);

    }


}
