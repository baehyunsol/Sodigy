use crate::error::ParseIntError;

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

    pub fn add_u32_mut(&mut self, n: u32) {
        if self.is_neg {
            todo!()
        }

        else {
            match self.nums[0].checked_add(n) {
                Some(n) => {
                    self.nums[0] = n;
                },
                None => {
                    let mut self_data = v32_to_v64(&self.nums);
                    self_data[0] += n as u64;

                    self.nums = v64_to_v32(self_data);
                },
            }
        }

        #[cfg(test)] { self.assert_valid(); }
    }

    pub fn mul_u32_mut(&mut self, n: u32) {
        let mut carry = 0;

        for i in 0..self.nums.len() {
            match self.nums[i].checked_mul(n) {
                Some(n) => match n.checked_add(carry as u32) {
                    Some(n) => {
                        self.nums[i] = n;
                        carry = 0;
                    },
                    None => {
                        self.nums[i] = ((n as u64 + carry) & 0xffff_ffff) as u32;
                        carry = (n as u64 + carry) >> 32;
                    },
                },
                None => {
                    let curr = self.nums[i] as u64 * n as u64 + carry;
                    carry = curr >> 32;
                    self.nums[i] = (curr & 0xffff_ffff) as u32;
                },
            }
        }

        if carry > 0 {
            self.nums.push(carry as u32);
        }

        remove_suffix_0(&mut self.nums);
        #[cfg(test)] { self.assert_valid(); }
    }

    pub fn parse_positive_hex(bytes: &[u8]) -> Result<BigInt, ParseIntError> {
        todo!()
    }

    pub fn parse_positive_decimal(bytes: &[u8]) -> Result<BigInt, ParseIntError> {
        let mut result = BigInt::zero();
        let mut buffer = 0;
        let mut counter = 0;

        for b in bytes.iter() {
            match b {
                b'0'..=b'9' => {
                    buffer *= 10;
                    buffer += (*b - b'0') as u32;
                    counter += 1;

                    if counter == 8 {
                        result.mul_u32_mut(100_000_000);
                        result.add_u32_mut(buffer);
                        counter = 0;
                        buffer = 0;
                    }
                },
                _ => {
                    return Err(ParseIntError);
                },
            }
        }

        if counter != 0 {
            result.mul_u32_mut(10u32.pow(counter));
            result.add_u32_mut(buffer);
        }

        Ok(result)
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
