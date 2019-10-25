use point::JacobianPoint;
use scalar::ScalarN;
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
    use context::CONTEXT;
    use rug::Integer;
    use scalar::ScalarN;
    use std::collections::BinaryHeap;
    use std::ops::Add;
    use util::term::Term;

    #[test]
    fn test_heap() {
        let mut heap = BinaryHeap::new();
        let one = ScalarN(Integer::from(1));
        let two = one.clone().add(one.clone());
        let three = two.clone().add(one.clone());
        let five = two.clone().add(three.clone());
        let a = Term {
            coeff: one.clone(),
            point: CONTEXT.G_jacobian.clone(),
        };
        heap.push(a);
        let a = Term {
            coeff: two,
            point: CONTEXT.G_jacobian.clone(),
        };
        heap.push(a);
        let a = Term {
            coeff: five,
            point: CONTEXT.G_jacobian.clone(),
        };
        heap.push(a);
        let a = Term {
            coeff: three,
            point: CONTEXT.G_jacobian.clone(),
        };
        heap.push(a);
    }
}
