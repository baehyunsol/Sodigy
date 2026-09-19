use super::{
    cmp::{cmp_ubi, lt_ubi},
    remove_suffix_0,
    v64_to_v32,
};
use std::cmp::Ordering;


pub(crate) fn gcd_ubi(lhs: &[u32], rhs: &[u32]) -> Vec<u32> {
    if rhs == [0] {
        lhs.to_vec()
    }

    else {
        gcd_ubi(rhs, &rem_ubi(lhs, rhs))
    }
}

pub fn neg_bi(rhs_neg: bool, rhs: &[u32]) -> (bool, Vec<u32>) {
    if rhs == [0] {
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
            let (abs_diff, is_neg) = sub_ubi_worker(lhs, rhs);
            (lhs_neg ^ is_neg, abs_diff)
        },
    }
}

/// It assumes that lhs is greater than or equal to rhs.
pub fn sub_ubi(lhs: &[u32], rhs: &[u32]) -> Vec<u32> {
    // println!("{lhs:?} - {rhs:?}");
    sub_ubi_worker(lhs, rhs).0
}

fn sub_ubi_worker(lhs: &[u32], rhs: &[u32]) -> (Vec<u32>, bool) {
    if lhs.len() < rhs.len() {
        return (sub_ubi_worker(rhs, lhs).0, true);
    }

    let mut result: Vec<u64> = vec![0; lhs.len()];
    let mut is_rhs_greater: Option<bool> = None;

    for i in 0..lhs.len() {
        let i = lhs.len() - 1 - i;

        match (lhs[i], rhs.get(i), is_rhs_greater) {
            (l, None, _) => {
                is_rhs_greater = Some(false);
                result[i] = l as u64 + u32::MAX as u64;
            },
            (l, Some(r), None) => {
                match l.cmp(r) {
                    Ordering::Less => {
                        is_rhs_greater = Some(true);
                    },
                    Ordering::Equal => {},
                    Ordering::Greater => {
                        is_rhs_greater = Some(false);
                    },
                }

                result[i] = l as u64 + (!*r) as u64;
            },
            (l, Some(r), Some(_)) => {
                result[i] = l as u64 + (!*r) as u64;
            },
        }
    }

    result[0] += 1;
    let mut result = v64_to_v32(result);

    if is_rhs_greater == Some(true) {
        for n in result.iter_mut() {
            *n = !*n;
        }

        remove_suffix_0(&mut result);
        (add_ubi(&result, &[1]), true)
    } else {
        result.pop().unwrap();
        remove_suffix_0(&mut result);
        (result, false)
    }
}

pub fn mul_bi(
    lhs_neg: bool,
    lhs: &[u32],
    rhs_neg: bool,
    rhs: &[u32],
) -> (bool, Vec<u32>) {
    // println!("{:?} * {:?}", (lhs_neg, lhs), (rhs_neg, rhs));
    let m = mul_ubi(lhs, rhs);

    match &m[..] {
        [0] => (false, m),
        _ => (lhs_neg ^ rhs_neg, m),
    }
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

// Sodigy uses truncated division.
pub fn div_bi(
    lhs_neg: bool,
    lhs: &[u32],
    rhs_neg: bool,
    rhs: &[u32],
) -> (bool, Vec<u32>) {
    // println!("{:?} / {:?}", (lhs_neg, lhs), (rhs_neg, rhs));
    let d = div_ubi(lhs, rhs);

    match &d[..] {
        [0] => (false, d),
        _ => (lhs_neg ^ rhs_neg, d),
    }
}

pub fn div_ubi(lhs: &[u32], rhs: &[u32]) -> Vec<u32> {
    // println!("{lhs:?} / {rhs:?}");
    match (lhs.len(), rhs.len()) {
        (l, r) if l < r => vec![0],
        (0..3, 0..3) => {
            let lhs: u64 = lhs[0] as u64 | ((*lhs.get(1).unwrap_or(&0) as u64) << 32);
            let rhs: u64 = rhs[0] as u64 | ((*rhs.get(1).unwrap_or(&0) as u64) << 32);
            let n = lhs / rhs;

            match n {
                0..=0xffff_ffff => vec![n as u32],
                _ => vec![(n & 0xffff_ffff) as u32, (n >> 32) as u32],
            }
        },
        (_, 1) => {
            let mut carry = 0;
            let rhs = rhs[0] as u64;
            let mut lhs = lhs.to_vec();

            for n in lhs.iter_mut().rev() {
                let curr = *n as u64 + carry;
                *n = (curr / rhs) as u32;
                carry = (curr % rhs) << 32;
            }

            remove_suffix_0(&mut lhs);
            lhs
        },
        (0..5, 0..5) => {
            let lhs: u128 = lhs[0] as u128 | ((*lhs.get(1).unwrap_or(&0) as u128) << 32) | ((*lhs.get(2).unwrap_or(&0) as u128) << 64) | ((*lhs.get(3).unwrap_or(&0) as u128) << 96);
            let rhs: u128 = rhs[0] as u128 | ((*rhs.get(1).unwrap_or(&0) as u128) << 32) | ((*rhs.get(2).unwrap_or(&0) as u128) << 64) | ((*rhs.get(3).unwrap_or(&0) as u128) << 96);
            let n = lhs / rhs;

            match n {
                0..=0xffff_ffff => vec![n as u32],
                ..=0xffff_ffff_ffff_ffff => vec![(n & 0xffff_ffff) as u32, (n >> 32) as u32],

                // rhs.len() is at least 2
                _ => vec![(n & 0xffff_ffff) as u32, ((n >> 32) & 0xffff_ffff) as u32, (n >> 64) as u32],
            }
        },
        (l, r) if l == r => {
            let lhs_hi = lhs[lhs.len() - 2] as u64 | ((lhs[lhs.len() - 1] as u64) << 32);
            let rhs_hi = rhs[rhs.len() - 2] as u64 | ((rhs[rhs.len() - 1] as u64) << 32);

            match lhs_hi.cmp(&rhs_hi) {
                Ordering::Less => vec![0],
                Ordering::Equal => match cmp_ubi(lhs, rhs) {
                    Ordering::Greater | Ordering::Equal => vec![1],
                    Ordering::Less => vec![0],
                },
                Ordering::Greater => {
                    // We add 1 to `rhs_hi` because `approx` has to be less
                    // than or equal to `lhs / rhs`.
                    let approx = lhs_hi / (rhs_hi + 1);
                    let approx = match approx {
                        0..=0xffff_ffff => vec![approx as u32],
                        _ => vec![(approx & 0xffff_ffff) as u32, (approx >> 32) as u32],
                    };

                    // lhs / rhs = approx + (lhs - rhs * approx) / rhs
                    add_ubi(&approx, &div_ubi(&sub_ubi(lhs, &mul_ubi(rhs, &approx)), rhs))
                },
            }
        },
        // l > r && r > 1
        _ => {
            // if we take rhs[-2..], it might recurse infinitely
            // if we take rhs[-1..], it'd be problematic if rhs[-1] is 1 or 2
            let lhs_ilog2 = lhs.last().unwrap().ilog2() + lhs.len() as u32 * 32 /* - 32 */;
            let rhs_ilog2 = rhs.last().unwrap().ilog2() + rhs.len() as u32 * 32 /* - 32 */;

            // We subtract 1 at the end because `approx` has to be less
            // than or equal to `lhs / rhs`.
            let approx = shl_ubi(&[1], lhs_ilog2 - rhs_ilog2 - 1);

            // lhs / rhs = approx + (lhs - rhs * approx) / rhs
            add_ubi(&approx, &div_ubi(&sub_ubi(lhs, &mul_ubi(rhs, &approx)), rhs))
        },
    }
}

// Sodigy uses truncated division.
pub fn rem_bi(
    lhs_neg: bool,
    lhs: &[u32],
    rhs_neg: bool,
    rhs: &[u32],
) -> (bool, Vec<u32>) {
    match (lhs.len(), rhs.len()) {
        (0..4, 0..4) => {
            let mut lhs: i128 = lhs[0] as i128 | ((*lhs.get(1).unwrap_or(&0) as i128) << 32) | ((*lhs.get(2).unwrap_or(&0) as i128) << 64);
            let mut rhs: i128 = rhs[0] as i128 | ((*rhs.get(1).unwrap_or(&0) as i128) << 32) | ((*rhs.get(2).unwrap_or(&0) as i128) << 64);

            if lhs_neg {
                lhs *= -1;
            }

            if rhs_neg {
                rhs *= -1;
            }

            let mut n = lhs % rhs;
            let is_neg = n < 0;
            n = n.abs();

            match n {
                0..=0xffff_ffff => (is_neg, vec![n as u32]),
                ..=0xffff_ffff_ffff_ffff => (is_neg, vec![(n & 0xffff_ffff) as u32, (n >> 32) as u32]),
                _ => (is_neg, vec![(n & 0xffff_ffff) as u32, ((n >> 32) & 0xffff_ffff) as u32, (n >> 64) as u32]),
            }
        },
        // (a + b * BIG + c * BIG * BIG + ...) % d = (a + (b % d) * (BIG % d) + (c % d) * (BIG % d) * (BIG % d) + ...) % d
        (_, 1) => {
            let d = rhs[0] as u64;
            let mut accum = lhs[0] as u64;
            let mut pow_rem = 0x1_0000_0000 % d;
            let big_rem = 0x1_0000_0000 % d;

            for b in lhs[1..].iter() {
                accum += (*b as u64 * pow_rem) % d;
                pow_rem = (pow_rem * big_rem) % d;
            }

            (lhs_neg, vec![(accum % d) as u32])
        },
        _ => {
            let (qn, q) = div_bi(lhs_neg, lhs, rhs_neg, rhs);
            let (ntn, nt) = mul_bi(qn, &q, rhs_neg, rhs);
            sub_bi(lhs_neg, lhs, ntn, &nt)
        },
    }
}

pub fn rem_ubi(lhs: &[u32], rhs: &[u32]) -> Vec<u32> {
    match (lhs.len(), rhs.len()) {
        (0..3, 0..3) => {
            let lhs: u64 = lhs[0] as u64 | ((*lhs.get(1).unwrap_or(&0) as u64) << 32);
            let rhs: u64 = rhs[0] as u64 | ((*rhs.get(1).unwrap_or(&0) as u64) << 32);
            let n = lhs % rhs;

            match n {
                0..=0xffff_ffff => vec![n as u32],
                _ => vec![(n & 0xffff_ffff) as u32, (n >> 32) as u32],
            }
        },
        // (a + b * BIG + c * BIG * BIG + ...) % d = (a + (b % d) * (BIG % d) + (c % d) * (BIG % d) * (BIG % d) + ...) % d
        (_, 1) => {
            let d = rhs[0] as u64;
            let mut accum = lhs[0] as u64;
            let mut pow_rem = 0x1_0000_0000 % d;
            let big_rem = 0x1_0000_0000 % d;

            for b in lhs[1..].iter() {
                accum += (*b as u64 * pow_rem) % d;
                pow_rem = (pow_rem * big_rem) % d;
            }

            vec![(accum % d) as u32]
        },
        _ => {
            sub_ubi(lhs, &mul_ubi(&div_ubi(lhs, rhs), rhs))
        },
    }
}

pub fn shl_ubi(lhs: &[u32], rhs: u32) -> Vec<u32> {
    match rhs {
        0 => lhs.to_vec(),
        1..32 => {
            let mut result = vec![0; lhs.len() + 1];

            for (i, lhs) in lhs.iter().enumerate() {
                let tail = (lhs & ((1 << (32 - rhs)) - 1)) << rhs;
                let head = lhs >> (32 - rhs);
                result[i] |= tail;
                result[i + 1] |= head;
            }

            remove_suffix_0(&mut result);
            result
        },
        32 => {
            let mut result = lhs.to_vec();
            result.insert(0, 0);
            result
        },
        33..64 => {
            let mut result = vec![0; lhs.len() + 2];

            for (i, lhs) in lhs.iter().enumerate() {
                let tail = (lhs & ((1 << (64 - rhs)) - 1)) << (rhs - 32);
                let head = lhs >> (64 - rhs);
                result[i + 1] |= tail;
                result[i + 2] |= head;
            }

            remove_suffix_0(&mut result);
            result
        },
        64 => {
            let mut result = lhs.to_vec();
            result.insert(0, 0);
            result.insert(0, 0);
            result
        },
        _ => shl_ubi(&shl_ubi(lhs, 64), rhs - 64),
    }
}

pub fn shr_ubi(lhs: &[u32], rhs: u32) -> Vec<u32> {
    match (lhs.len(), rhs) {
        (_, 0) => lhs.to_vec(),
        (_, 1..32) => {
            let mut result = vec![0; lhs.len()];
            result[0] = lhs[0] >> rhs;

            if lhs.len() > 1 {
                for (i, lhs) in lhs[1..].iter().enumerate() {
                    let tail = lhs << (32 - rhs);
                    let head = lhs >> rhs;
                    result[i] |= tail;
                    result[i + 1] |= head;
                }

                remove_suffix_0(&mut result);
            }

            result
        },
        (1, _) => vec![0],
        (_, 32) => lhs[1..].to_vec(),
        (_, 33..64) => {
            let mut result = vec![0; lhs.len() - 1];
            result[0] = lhs[1] >> (rhs - 32);

            for (i, lhs) in lhs[2..].iter().enumerate() {
                let tail = lhs << (64 - rhs);
                let head = lhs >> (rhs - 32);
                result[i] |= tail;
                result[i + 1] |= head;
            }

            remove_suffix_0(&mut result);
            result
        },
        (2, _) => vec![0],
        (_, 64) => lhs[2..].to_vec(),
        _ => shr_ubi(&shr_ubi(lhs, 64), rhs - 64),
    }
}

pub fn and_ubi(lhs: &[u32], rhs: &[u32]) -> Vec<u32> {
    let len = lhs.len().max(rhs.len());
    (0..len).map(
        |i| lhs.get(i).unwrap_or(&0) & rhs.get(i).unwrap_or(&0)
    ).collect()
}

pub fn or_ubi(lhs: &[u32], rhs: &[u32]) -> Vec<u32> {
    let len = lhs.len().max(rhs.len());
    (0..len).map(
        |i| lhs.get(i).unwrap_or(&0) | rhs.get(i).unwrap_or(&0)
    ).collect()
}
