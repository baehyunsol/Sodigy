use super::mul_ubi;

pub fn ilog2_ubi(ns: &[u32]) -> u32 {
    match ns.last() {
        Some(n) => n.ilog2() + (ns.len() - 1) as u32 * 32,
        None => unreachable!(),
    }
}

pub fn powi_ubi(n: &[u32], mut p: u32) -> Vec<u32> {
    if p == 0 {
        return vec![1];
    }

    let mut pows = vec![n.to_vec()];
    let mut exps = vec![1];

    while let (Some(curr_pow), Some(curr_exp)) = (pows.last(), exps.last()) {
        if *curr_exp * 2 > p {
            break;
        }

        pows.push(mul_ubi(curr_pow, curr_pow));
        exps.push(*curr_exp * 2);
    }

    let mut result = vec![1];

    while let (Some(curr_pow), Some(curr_exp)) = (pows.pop(), exps.pop()) {
        if curr_exp <= p {
            p -= curr_exp;
            result = mul_ubi(&curr_pow, &result);
        }
    }

    result
}
