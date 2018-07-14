use scalar::ScalarN;
use point::JacobianPoint;
use std::cmp::Ordering;

#[derive(Debug)]
pub struct ProductTerm {
    pub coeff: ScalarN,
    pub point: JacobianPoint,
}


impl Ord for ProductTerm {
    fn cmp(&self, other: &ProductTerm) -> Ordering {
        self.coeff.0.cmp(&other.coeff.0)
    }
}

impl PartialOrd for ProductTerm {
    fn partial_cmp(&self, other: &ProductTerm) -> Option<Ordering> {
        Some(self.cmp(other))
    }
}

impl PartialEq for ProductTerm {
    fn eq(&self, other: &ProductTerm) -> bool {
        self.coeff == other.coeff
    }
}

impl Eq for ProductTerm {}

#[cfg(test)]
mod tests {
    use std::collections::BinaryHeap;
    use scalar::ScalarN;
    use optimized_product::ProductTerm;
    use point::JacobianPoint;
    use num_bigint::BigUint;
    use num_traits::One;
    use context::CONTEXT;
    use std::ops::Sub;
    use std::ops::Add;

    #[test]
    fn test_musig() {
        let mut heap = BinaryHeap::new();
        let one = ScalarN(BigUint::one());
        let two = one.clone().add(one.clone());
        let three = two.clone().add(one.clone());
        let five = two.clone().add(three.clone());
        let a = ProductTerm{coeff: one.clone(), point: CONTEXT.G_jacobian.clone() };
        heap.push(a);
        let a = ProductTerm{coeff: two, point: CONTEXT.G_jacobian.clone() };
        heap.push(a);
        let a = ProductTerm{coeff: five, point: CONTEXT.G_jacobian.clone() };
        heap.push(a);
        let a = ProductTerm{coeff: three, point: CONTEXT.G_jacobian.clone() };
        heap.push(a);



    }
}

