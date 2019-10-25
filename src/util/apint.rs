use apint::ApInt;
use apint::Width;
use std::ops::Mul;
use std::ops::Sub;

#[derive(Clone, Debug)]
pub struct JacobianPointApInt {
    pub x: ApInt,
    pub y: ApInt,
    pub z: ApInt,
}

#[derive(Clone, Debug)]
pub struct PointApInt {
    pub x: ApInt,
    pub y: ApInt,
}

pub fn mixed_point_add_apint(
    p1: Option<&JacobianPointApInt>,
    p2: Option<&PointApInt>,
) -> Option<JacobianPointApInt> {
    match (p1, p2) {
        (None, None) => None,
        (Some(_p1), None) => None,
        (None, Some(_p2)) => None,
        (Some(p1), Some(p2)) => {
            println!("{:?} ", p1.z.width());
            println!("{:?} ", p1.z.clone().width());
            let p1_z_pow2 = p1.z.clone().mul(&p1.z);

            let u1 = &p1.x;
            let u2 = p2.x.clone().mul(&p1_z_pow2);

            let s1 = &p1.y;
            let s2 = p1_z_pow2.mul(&p1.z).mul(&p2.y);

            if *u1 == u2 {
                if *s1 == s2 {
                    return Some(p1.to_owned()); //TODO
                } else {
                    return None;
                }
            }
            let h = u2.sub(&u1);
            let h_pow2 = h.clone().mul(&h);
            //println!("capacity {}", h_pow2.0.capacity());

            let h_pow3 = h_pow2.clone().mul(&h);
            let r = s2.sub(&s1);
            let r_pow2 = r.clone().mul(&r);

            let x3 = r_pow2
                .sub(&h_pow3)
                .sub(&ApInt::from_u64(2).mul(&u1).mul(&h_pow2));

            let u1_mul_h_pow2 = h_pow2.mul(&u1);

            let y3 = r.mul(&u1_mul_h_pow2.sub(&x3)).sub(&h_pow3.mul(&s1));

            let z3 = h.mul(&p1.z);
            Some(JacobianPointApInt {
                x: x3,
                y: y3,
                z: z3,
            })
        }
    }
}

#[allow(dead_code)]
fn apint_to_bytes_le(mut apint: ApInt) -> Vec<u8> {
    let mut bytes = Vec::with_capacity(apint.width().to_usize() / 8);
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
    use super::*;
    use apint::ApInt;
    use apint::BitWidth;
    use num_bigint::BigUint;
    use num_traits::FromPrimitive;
    use rand::thread_rng;
    use rand::Rng;

    #[test]
    fn test_apint() {
        let apint = ApInt::from_u64(1000);
        let biguint = BigUint::from_u32(1000).unwrap();

        assert_eq!(biguint.to_bytes_le(), apint_to_bytes_le(apint.clone()));

        let x = apint.sub(&ApInt::from_u64(1000));
        assert!(x.is_zero());

        let apint = ApInt::all_unset(BitWidth::new(512).unwrap());

        let b = ApInt::all_set(BitWidth::new(512).unwrap());
        let apint = apint + &b;
        //println!("{:?}", apint);

        /*
                let rnd =  ApInt::from_u64( thread_rng().gen() );

                b.assign(&rnd);
                let apint = apint + &b;
                println!("{:?}", apint);
                let apint = apint * &b;
                println!("{:?}", apint);
                let apint = apint * &b;
                println!("{:?}", apint);

                let a = JacobianPointApInt{
                    x: ApInt::random_with_width(BitWidth::new(256).unwrap()),
                    y: ApInt::random_with_width(BitWidth::new(256).unwrap()),
                    z: ApInt::random_with_width(BitWidth::new(256).unwrap()),
                };

                let b = PointApInt{
                    x: ApInt::random_with_width(BitWidth::new(256).unwrap()),
                    y: ApInt::random_with_width(BitWidth::new(256).unwrap()),
                };
        */
        //let option = mixed_point_add_apint(Some(&a), Some(&b) );
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
