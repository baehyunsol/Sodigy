use super::BigInt;
use crate::{div_ubi, gt_ubi, rem_ubi};

impl From<u64> for BigInt {
    fn from(n: u64) -> BigInt {
        let nums = match n {
            0..4294967296 => vec![n as u32],
            _ => vec![(n & 0xffff_ffff) as u32, (n >> 32) as u32],
        };
        BigInt {
            is_neg: false,
            nums,
        }
    }
}

impl From<i64> for BigInt {
    fn from(n: i64) -> BigInt {
        let mut abs = BigInt::from((n as i128).abs() as u64);
        abs.is_neg = n < 0;
        abs
    }
}

impl TryFrom<&BigInt> for u64 {
    type Error = ();

    fn try_from(n: &BigInt) -> Result<u64, ()> {
        if n.is_neg {
            Err(())
        }

        else {
            match &n.nums[..] {
                [x] => Ok(*x as u64),
                [x, y] => Ok(*x as u64 | ((*y as u64) << 32)),
                _ => Err(()),
            }
        }
    }
}

impl TryFrom<&BigInt> for i64 {
    type Error = ();

    fn try_from(n: &BigInt) -> Result<i64, ()> {
        let nu64 = match &n.nums[..] {
            [x] => *x as u64,
            [x, y] => *x as u64 | ((*y as u64) << 32),
            _ => {
                return Err(());
            },
        };
        let mut ni128 = nu64 as i128;

        if n.is_neg {
            ni128 *= -1;
        }

        match i64::try_from(ni128) {
            Ok(n) => Ok(n),
            Err(_) => Err(()),
        }
    }
}

pub fn bi_to_string(neg: bool, ns: &[u32]) -> String {
    let n = ubi_to_string(ns);

    if neg {
        format!("-{n}")
    }

    else {
        n
    }
}

pub fn ubi_to_string(ns: &[u32]) -> String {
    let mut ns = ns.to_vec();
    let mut digits = vec![];
    let million = [1_000_000];

    while gt_ubi(&ns, &million) {
        let r = rem_ubi(&ns, &million)[0];
        ns = div_ubi(&ns, &million);
        digits.push(format!("{r:06}"));
    }

    digits.push(ns[0].to_string());
    digits.into_iter().rev().collect::<Vec<_>>().concat()
}

#[cfg(test)]
mod tests {
    use crate::{BigInt, ubi_to_string};

    #[test]
    fn to_string_test() {
        let samples: Vec<&[u8]> = vec![
            b"0", b"1", b"2", b"3", b"4",
            b"10", b"11", b"12", b"13",
            b"100", b"101", b"102", b"103",
            b"999999", b"1000000", b"1000001",
            b"1999999", b"2000000", b"2000001",
            b"2718281828459045",
            b"31415926535897932384626",
            b"3162277660168379331998",
            b"9999999999999999999999",
        ];

        for s in samples {
            let n = BigInt::parse_positive_decimal(s).unwrap();
            let ss = ubi_to_string(&n.nums);
            assert_eq!(s, ss.as_bytes());
        }
    }
}
