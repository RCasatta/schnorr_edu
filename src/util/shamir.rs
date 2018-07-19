use scalar::ScalarN;
use point::JacobianPoint;
use point::jacobian_point_add;
use point::jacobian_point::jacobian_point_double;
use rug::Integer;

#[allow(non_snake_case)]
pub fn shamirs_trick(k : ScalarN, P: JacobianPoint, l: ScalarN, Q : JacobianPoint) -> JacobianPoint {
    //precompute
    let mut precomputed : Vec<Option<JacobianPoint>> = Vec::with_capacity(4);
    precomputed.push(None);
    precomputed.push(Some(Q.clone()));
    precomputed.push(Some(P.clone()));
    precomputed.push(jacobian_point_add( Some(&Q), Some(&P)));

    let mut acc : Option<JacobianPoint> = None;
    let mut exponent : Integer = Integer::from(1) << 255;

    loop {
        let a1 : Integer = (&k.0 & &exponent).into();
        let a = (a1 != 0) as usize;
        let b1 : Integer = (&l.0 & &exponent).into();
        let b = (b1 != 0) as usize;
        let index = a * 2 + b;
        let current = precomputed[index].to_owned();

        if acc.is_some() {
            acc = jacobian_point_double(&acc.unwrap());
        }
        if current.is_some() {
            acc = jacobian_point_add(acc.as_ref(), current.as_ref());
        }
        exponent = exponent >> 1;
        if exponent == 0 {
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
    use point::jacobian_point_add;
    use point::generator_mul;
    use rug::Integer;

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
        let two_scalar_n = ScalarN(Integer::from(2));

        let mut exponent : Integer = Integer::from(1)<<255;
        let mut acc : Option<Point> = None;

        loop {
            let val : Integer = (&k.0 & &exponent).into();
            let a : bool = val != 0;
            if acc.is_some() {
                acc = point_mul(acc.unwrap(),two_scalar_n.clone());
            }
            if a {
                acc = point_add(acc, Some(P.clone()));
            }
            exponent = exponent >> 1;
            if exponent == 0 {
                break;
            }
        }

        assert_eq!( P.mul(&k), acc.unwrap());

    }
}