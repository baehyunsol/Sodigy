use crate::error::ParseIntError;

pub mod op;
pub mod cmp;
mod convert;

use op::{add_ubi, mul_ubi, shl_ubi};

#[derive(Clone, Debug)]
pub struct BigInt {
    pub is_neg: bool,
    pub nums: Vec<u32>,
}

// TODO: create modules
impl BigInt {
    // Zero must be represented in this way. Only this way.
    pub fn zero() -> Self {
        BigInt {
            is_neg: false,
            nums: vec![0],
        }
    }

    pub fn one() -> Self {
        BigInt {
            is_neg: false,
            nums: vec![1],
        }
    }

    pub fn is_one(&self) -> bool {
        !self.is_neg && &self.nums == &[1]
    }

    pub fn parse_positive_hex(bytes: &[u8]) -> Result<BigInt, ParseIntError> {
        let mut result = vec![0];
        let mut buffer = 0;
        let mut counter = 0;

        for b in bytes.iter() {
            let n = match b {
                b'0'..=b'9' => (*b - b'0') as u32,
                b'a'..=b'f' => (*b - b'a' + 10) as u32,
                b'A'..=b'F' => (*b - b'A' + 10) as u32,
                _ => {
                    return Err(ParseIntError);
                },
            };

            buffer <<= 4;
            buffer |= n;
            counter += 1;

            if counter == 6 {
                result = shl_ubi(&result, 24);
                result = add_ubi(&result, &[buffer]);
                counter = 0;
                buffer = 0;
            }
        }

        if counter != 0 {
            result = shl_ubi(&result, counter << 2);
            result = add_ubi(&result, &[buffer]);
        }

        Ok(BigInt {
            is_neg: false,
            nums: result,
        })
    }

    pub fn parse_positive_decimal(bytes: &[u8]) -> Result<BigInt, ParseIntError> {
        let mut result = vec![0];
        let mut buffer = 0;
        let mut counter = 0;

        for b in bytes.iter() {
            match b {
                b'0'..=b'9' => {
                    buffer *= 10;
                    buffer += (*b - b'0') as u32;
                    counter += 1;
                },
                _ => {
                    return Err(ParseIntError);
                },
            }

            if counter == 8 {
                result = mul_ubi(&result, &[100_000_000]);
                result = add_ubi(&result, &[buffer]);
                counter = 0;
                buffer = 0;
            }
        }

        if counter != 0 {
            result = mul_ubi(&result, &[10u32.pow(counter)]);
            result = add_ubi(&result, &[buffer]);
        }

        Ok(BigInt {
            is_neg: false,
            nums: result,
        })
    }
}

pub fn remove_suffix_0(vec: &mut Vec<u32>) {
    while vec.len() > 1 && vec[vec.len() - 1] == 0 {
        vec.pop().unwrap();
    }
}

pub fn v32_to_v64(v32: &Vec<u32>) -> Vec<u64> {
    #[cfg(test)] { assert!(v32.len() > 0); }
    v32.iter().map(|n| *n as u64).collect()
}

pub fn v64_to_v32(mut v64: Vec<u64>) -> Vec<u32> {
    #[cfg(test)] { assert!(v64.len() > 0); }

    for i in 0..(v64.len() - 1) {
        if v64[i] >= (1 << 32) {
            v64[i + 1] += v64[i] >> 32;
            v64[i] &= 0xffff_ffff;
        }
    }

    let v64_len = v64.len() - 1;

    if v64[v64_len] >= (1 << 32) {
        v64.push(v64[v64_len] >> 32);
        v64[v64_len] &= 0xffff_ffff;
    }

    #[cfg(test)] { assert!(v64.iter().all(|n| *n < (1 << 32))); }
    v64.into_iter().map(|n| n as u32).collect()
}
