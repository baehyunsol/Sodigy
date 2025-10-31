use crate::error::ParseIntError;

#[derive(Clone, Debug)]
pub struct BigInt {
    pub is_neg: bool,
    pub nums: Vec<u32>,
}

impl BigInt {
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
            result.mul_u32_mut(10.pow(counter));
            result.add_u32_mut(buffer);
        }

        Ok(result)
    }
}
