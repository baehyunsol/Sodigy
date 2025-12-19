use super::BigInt;

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
