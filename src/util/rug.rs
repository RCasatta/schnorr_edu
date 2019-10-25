use context::CONTEXT;
use data_encoding::HEXLOWER;
use rug::Integer;
use std::ops::Rem;

pub fn integer_from_bytes(val: &[u8]) -> Integer {
    Integer::from_str_radix(&HEXLOWER.encode(val), 16).unwrap()
}

pub fn mul_and_rem(mut a512: Integer, b256: &Integer) -> Integer {
    a512 *= b256;
    a512.rem(&CONTEXT.p.0)
}

#[cfg(test)]
mod tests {
    use context::CONTEXT;
    use num_bigint::BigUint;
    use num_traits::Num;
    use rand::thread_rng;
    use rand::Rng;
    use rug::Assign;
    use rug::Integer;
    use util::rug::integer_from_bytes;
    use util::rug::mul_and_rem;

    #[test]
    fn test_mul_and_rem() {
        let mut bytes = [0; 32];
        thread_rng().fill(&mut bytes);
        let a = integer_from_bytes(&bytes);
        thread_rng().fill(&mut bytes);
        let b = integer_from_bytes(&bytes);
        let mut buffer = Integer::with_capacity(512);
        buffer.assign(&a);

        let val1 = mul_and_rem(buffer, &b);
        let val2 = (a * &b) % &CONTEXT.p.0;
        assert_eq!(val1, val2);
    }

    #[test]
    fn test_rug() {
        let mut rng = thread_rng();
        let val: u64 = rng.gen();
        let integer = Integer::from(val);

        assert_eq!(val, integer.to_u64().unwrap());

        let string = integer.to_string_radix(10);
        assert_eq!(format!("{}", val), string);

        let result = BigUint::from_str_radix(&string, 10).unwrap();
        assert_eq!(format!("{}", result), string);

        let mut a = Integer::from(10);
        let b = Integer::from(120);
        a += &b;

        //integer.assign(a);
        assert_eq!(a, 130);
    }
}
