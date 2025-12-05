use std::cmp::Ordering;

// lhs < rhs
pub fn lt_bi(
    lhs_neg: bool,
    lhs: &[u32],
    rhs_neg: bool,
    rhs: &[u32],
) -> bool {
    // println!("{:?} < {:?}", (lhs_neg, lhs), (rhs_neg, rhs));
    if lhs_neg != rhs_neg {
        lhs_neg
    }

    else {
        match cmp_ubi(lhs, rhs) {
            Ordering::Equal => false,
            // -3 > -4
            Ordering::Less if lhs_neg => false,
            // 3 < 4
            Ordering::Less => true,
            // -4 < -3
            Ordering::Greater if lhs_neg => true,
            // 4 > 3
            Ordering::Greater => false,
        }
    }
}

pub fn lt_ubi(lhs: &[u32], rhs: &[u32]) -> bool {
    if lhs.len() < rhs.len() {
        true
    }

    else if lhs.len() > rhs.len() {
        false
    }

    else {
        for i in 1..(lhs.len() + 1) {
            if lhs[lhs.len() - i] < rhs[rhs.len() - i] {
                return true;
            }

            else if lhs[lhs.len() - i] > rhs[rhs.len() - i] {
                return false;
            }
        }

        false
    }
}

pub fn eq_bi(
    lhs_neg: bool,
    lhs: &[u32],
    rhs_neg: bool,
    rhs: &[u32],
) -> bool {
    println!("{:?} == {:?}", (lhs_neg, lhs), (rhs_neg, rhs));
    lhs_neg == rhs_neg && eq_ubi(lhs, rhs)
}

pub fn eq_ubi(lhs: &[u32], rhs: &[u32]) -> bool {
    lhs == rhs
}

pub fn gt_bi(
    lhs_neg: bool,
    lhs: &[u32],
    rhs_neg: bool,
    rhs: &[u32],
) -> bool {
    todo!()
}

pub fn gt_ubi(lhs: &[u32], rhs: &[u32]) -> bool {
    todo!()
}

pub fn cmp_ubi(lhs: &[u32], rhs: &[u32]) -> Ordering {
    if lhs.len() > rhs.len() {
        Ordering::Greater
    }

    else if lhs.len() < rhs.len() {
        Ordering::Less
    }

    else {
        for i in 1..(lhs.len() + 1) {
            if lhs[lhs.len() - i] < rhs[rhs.len() - i] {
                return Ordering::Less;
            }

            else if lhs[lhs.len() - i] > rhs[rhs.len() - i] {
                return Ordering::Greater;
            }
        }

        Ordering::Equal
    }
}
