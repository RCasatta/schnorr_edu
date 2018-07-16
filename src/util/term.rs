use scalar::ScalarN;
use point::JacobianPoint;
use std::cmp::Ordering;

#[derive(Debug)]
pub struct Term {
    pub coeff: ScalarN,
    pub point: JacobianPoint,
}


impl Ord for Term {
    fn cmp(&self, other: &Term) -> Ordering {
        self.coeff.0.cmp(&other.coeff.0)
    }
}

impl PartialOrd for Term {
    fn partial_cmp(&self, other: &Term) -> Option<Ordering> {
        Some(self.cmp(other))
    }
}

impl PartialEq for Term {
    fn eq(&self, other: &Term) -> bool {
        self.coeff == other.coeff
    }
}

impl Eq for Term {}

#[cfg(test)]
mod tests {
    use std::collections::BinaryHeap;
    use scalar::ScalarN;
    use util::term::Term;
    use num_bigint::BigUint;
    use num_traits::One;
    use context::CONTEXT;
    use std::ops::Add;

    #[test]
    fn test_heap() {
        let mut heap = BinaryHeap::new();
        let one = ScalarN(BigUint::one());
        let two = one.clone().add(one.clone());
        let three = two.clone().add(one.clone());
        let five = two.clone().add(three.clone());
        let a = Term {coeff: one.clone(), point: CONTEXT.G_jacobian.clone() };
        heap.push(a);
        let a = Term {coeff: two, point: CONTEXT.G_jacobian.clone() };
        heap.push(a);
        let a = Term {coeff: five, point: CONTEXT.G_jacobian.clone() };
        heap.push(a);
        let a = Term {coeff: three, point: CONTEXT.G_jacobian.clone() };
        heap.push(a);

    }
}

