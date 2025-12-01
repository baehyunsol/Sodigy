use super::{remove_suffix_0, v64_to_v32};

pub fn add_bi(
    lhs_neg: bool,
    lhs: &[u32],
    rhs_neg: bool,
    rhs: &[u32],
) -> (bool, Vec<u32>) {
    match (lhs_neg, rhs_neg) {
        (true, true) | (false, false) => (lhs_neg, add_ubi(lhs, rhs)),
        _ => todo!(),
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
    todo!()
}

pub fn sub_ubi(lhs: &[u32], rhs: &[u32]) -> Vec<u32> {
    todo!()
}

pub fn mul_bi(
    lhs_neg: bool,
    lhs: &[u32],
    rhs_neg: bool,
    rhs: &[u32],
) -> (bool, Vec<u32>) {
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
