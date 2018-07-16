use scalar::ScalarN;
use point::JacobianPoint;
use point::jacobian_point_add;
use num_bigint::BigUint;
use num_traits::One;
use num_traits::Zero;
use point::jacobian_point_mul;

#[allow(non_snake_case)]
pub fn shamirs_trick(k : ScalarN, P: JacobianPoint, l: ScalarN, Q : JacobianPoint) -> JacobianPoint {
    //precompute
    let mut precomputed : Vec<Option<JacobianPoint>> = Vec::with_capacity(4);
    precomputed.push(None);
    precomputed.push(Some(Q.clone()));
    precomputed.push(Some(P.clone()));
    precomputed.push(jacobian_point_add( Some(&Q), Some(&P)));

    let mut acc : Option<JacobianPoint> = None;
    let two = BigUint::one() + BigUint::one();
    let two_scalar_n = ScalarN(two);
    let mut exponent = BigUint::one()<<255;

    loop {
        let a = !(&k.0 & &exponent).is_zero() as usize;
        let b = !(&l.0 & &exponent).is_zero() as usize;
        let index = a * 2 + b;
        let current = precomputed[index].to_owned();

        if acc.is_some() {
            acc = jacobian_point_mul(&acc.unwrap(), &two_scalar_n);
        }
        if current.is_some() {
            acc = jacobian_point_add(acc.as_ref(), current.as_ref());
        }
        exponent >>= 1usize;
        if exponent.is_zero() {
            break;
        }
    }

    acc.unwrap()
}


#[cfg(test)]
mod tests {

    use point::Point;
    use context::CONTEXT;
    use scalar::ScalarN;
    use rand::thread_rng;
    use point::point_mul;
    use rand::Rng;
    use super::shamirs_trick;
    use point::point_add;
    use num_bigint::BigUint;
    use num_traits::One;
    use num_traits::Zero;
    use point::jacobian_point_add;
    use point::generator_mul;

    #[test]
    #[allow(non_snake_case)]
    fn test_shamir() {
        let mut rng = thread_rng();
        let P = CONTEXT.G_jacobian.clone();
        let Q = CONTEXT.G_jacobian.clone();
        //let Q = point_mul(P.clone(), rng.gen::<ScalarN>()).unwrap();
        let k = rng.gen::<ScalarN>();
        let l = rng.gen::<ScalarN>();
        let q = jacobian_point_add( generator_mul(&k).as_ref(),
                                    generator_mul( &l).as_ref()).unwrap();
        let r = shamirs_trick(k,P,l,Q);

        assert_eq!(r,q);
    }

    #[test]
    #[allow(non_snake_case)]
    fn test_double() {
        let mut rng = thread_rng();
        let P = CONTEXT.G.clone();
        let k = rng.gen::<ScalarN>();
        let two_scalar_n = ScalarN(BigUint::one() + BigUint::one());

        let mut exponent = BigUint::one()<<255;
        let mut acc : Option<Point> = None;

        loop {
            let a : bool = !(&k.0 & &exponent).is_zero();
            if acc.is_some() {
                acc = point_mul(acc.unwrap(),two_scalar_n.clone());
            }
            if a {
                acc = point_add(acc, Some(P.clone()));
            }
            exponent >>= 1usize;
            if exponent.is_zero() {
                break;
            }
        }

        assert_eq!( P.mul(&k), acc.unwrap());

    }
}