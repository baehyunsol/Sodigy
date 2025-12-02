use super::{
    cmp::lt_ubi,
    remove_suffix_0,
    v64_to_v32,
};

pub fn neg_bi(rhs_neg: bool, rhs: &[u32]) -> (bool, Vec<u32>) {
    if rhs == &[0] {
        (false, vec![0])
    }

    else {
        (!rhs_neg, rhs.to_vec())
    }
}

pub fn add_bi(
    lhs_neg: bool,
    lhs: &[u32],
    rhs_neg: bool,
    rhs: &[u32],
) -> (bool, Vec<u32>) {
    // println!("{:?} + {:?}", (lhs_neg, lhs), (rhs_neg, rhs));
    match (lhs_neg, rhs_neg) {
        (true, true) | (false, false) => (lhs_neg, add_ubi(lhs, rhs)),
        (true, false) | (false, true) => sub_bi(lhs_neg, lhs, !rhs_neg, rhs),
    }
}

pub fn add_ubi(lhs: &[u32], rhs: &[u32]) -> Vec<u32> {
    let mut result = Vec::with_capacity(lhs.len().max(rhs.len()));

    for i in 0..lhs.len().min(rhs.len()) {
        result.push(lhs[i] as u64 + rhs[i] as u64);
    }

    if lhs.len() > rhs.len() {
        for i in rhs.len()..lhs.len() {
            result.push(lhs[i] as u64);
        }
    }

    if rhs.len() > lhs.len() {
        for i in lhs.len()..rhs.len() {
            result.push(rhs[i] as u64);
        }
    }

    v64_to_v32(result)
}

pub fn sub_bi(
    lhs_neg: bool,
    lhs: &[u32],
    rhs_neg: bool,
    rhs: &[u32],
) -> (bool, Vec<u32>) {
    // println!("{:?} - {:?}", (lhs_neg, lhs), (rhs_neg, rhs));
    match (lhs_neg, rhs_neg) {
        (true, false) | (false, true) => (
            lhs_neg,
            add_ubi(lhs, rhs),
        ),
        _ => {
            let lhs_less = lt_ubi(lhs, rhs);
            let abs_diff = if lhs_less {
                sub_ubi(rhs, lhs)
            } else {
                sub_ubi(lhs, rhs)
            };

            if &abs_diff == &[0] {
                (false, abs_diff)
            } else {
                // lhs  -  rhs    lhs_less     lhs_neg      result
                //   3  -  4        true        false    ( true, abs_diff)
                //   4  -  3        false       false    (false, abs_diff)
                // (-3) - (-4)      true        true     (false, abs_diff)
                // (-4) - (-3)      false       true     ( true, abs_diff)
                (lhs_less ^ lhs_neg, abs_diff)
            }
        },
    }
}

/// It panics if `lhs < rhs`.
pub fn sub_ubi(lhs: &[u32], rhs: &[u32]) -> Vec<u32> {
    let mut result = lhs.to_vec();
    let mut carry = false;

    for i in 0..rhs.len() {
        if carry {
            if rhs[i] != u32::MAX && result[i] >= rhs[i] + 1 {
                result[i] -= rhs[i] + 1;
                carry = false;
            }

            else {
                result[i] = u32::MAX - (rhs[i] - result[i]);
            }
        }

        else {
            if result[i] >= rhs[i] {
                result[i] -= rhs[i];
            }

            else {
                result[i] = u32::MAX - (rhs[i] - result[i]) + 1;
                carry = true;
            }
        }
    }

    if carry {
        if result.len() <= rhs.len() {
            panic!();
        }

        for i in rhs.len()..result.len() {
            if result[i] > 0 {
                result[i] -= 1;
                break;
            }

            else {
                result[i] = u32::MAX;
            }
        }
    }

    remove_suffix_0(&mut result);
    result
}

pub fn mul_bi(
    lhs_neg: bool,
    lhs: &[u32],
    rhs_neg: bool,
    rhs: &[u32],
) -> (bool, Vec<u32>) {
    // println!("{:?} * {:?}", (lhs_neg, lhs), (rhs_neg, rhs));
    (lhs_neg ^ rhs_neg, mul_ubi(lhs, rhs))
}

pub fn mul_ubi(lhs: &[u32], rhs: &[u32]) -> Vec<u32> {
    let mut result = vec![0; lhs.len() + rhs.len()];

    for i in 0..lhs.len() {
        for j in 0..rhs.len() {
            let n = lhs[i] as u64 * rhs[j] as u64;
            result[i + j] += n & 0xffff_ffff;
            result[i + j + 1] += n >> 32;
        }
    }

    let mut result = v64_to_v32(result);
    remove_suffix_0(&mut result);
    result
}

pub fn div_bi(
    lhs_neg: bool,
    lhs: &[u32],
    rhs_neg: bool,
    rhs: &[u32],
) -> (bool, Vec<u32>) {
    todo!()
}

pub fn div_ubi(lhs: &[u32], rhs: &[u32]) -> Vec<u32> {
    todo!()
}

pub fn rem_bi(
    lhs_neg: bool,
    lhs: &[u32],
    rhs_neg: bool,
    rhs: &[u32],
) -> (bool, Vec<u32>) {
    todo!()
}

pub fn rem_ubi(lhs: &[u32], rhs: &[u32]) -> Vec<u32> {
    todo!()
}

pub fn shl_ubi(n: &[u32], other: u32) -> Vec<u32> {
    match other {
        0 => n.to_vec(),
        1..32 => {
            let mut result = vec![0; n.len() + 1];

            for (i, n) in n.iter().enumerate() {
                let tail = (n & ((1 << (32 - other)) - 1)) << other;
                let head = n >> (32 - other);
                result[i] |= tail;
                result[i + 1] |= head;
            }

            remove_suffix_0(&mut result);
            result
        },
        32 => todo!(),
        _ => todo!(),
    }
}

pub fn shr_ubi(n: &[u32], other: u32) -> Vec<u32> {
    todo!()
}
