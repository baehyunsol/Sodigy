use hmath::{BigInt, Ratio};
use std::rc::Rc;

mod fmt;

#[derive(Clone)]
pub struct SodigyData {
    pub value: SodigyDataValue,
    pub ty: SodigyDataType,
}

impl SodigyData {
    pub fn new_int(n: BigInt) -> Self {
        SodigyData {
            value: SodigyDataValue::BigInt(n),
            ty: SodigyDataType::Integer,
        }
    }

    pub fn new_ratio(denom: BigInt, numer: BigInt) -> Self {
        SodigyData {
            value: SodigyDataValue::Compound(vec![
                Rc::new(SodigyData {
                    value: SodigyDataValue::BigInt(denom),
                    ty: SodigyDataType::Integer,
                }),
                Rc::new(SodigyData {
                    value: SodigyDataValue::BigInt(numer),
                    ty: SodigyDataType::Integer,
                }),
            ]),
            ty: SodigyDataType::Ratio,
        }
    }

    pub fn new_func(index: usize) -> Self {
        SodigyData {
            value: SodigyDataValue::SmallInt(index as i32),
            ty: SodigyDataType::Func,
        }
    }

    pub fn new_char(c: char) -> Self {
        SodigyData {
            value: SodigyDataValue::SmallInt(c as i32),
            ty: SodigyDataType::Char,
        }
    }

    pub fn new_string(s: &[u8]) -> Self {
        let mut result = Vec::with_capacity(s.len());
        let mut curr_char = 0;

        for c in s.iter() {
            if *c < 128 {
                result.push(*c as i32);
            }

            else if *c < 192 {
                curr_char <<= 6;
                curr_char += (*c & 63) as i32;
                result.push(curr_char);
                curr_char = 0;
            }

            else if *c < 224 {
                curr_char = (*c & 31) as i32;
            }

            else if *c < 240 {
                curr_char = (*c & 15) as i32;
            }

            else if *c < 248 {
                curr_char = (*c & 7) as i32;
            }

            else {
                unreachable!();
            }
        }

        SodigyData {
            value: SodigyDataValue::Compound(
                result.into_iter().map(
                    |c| Rc::new(SodigyData {
                        value: SodigyDataValue::SmallInt(c),
                        ty: SodigyDataType::Char,
                    })
                ).collect()
            ),
            ty: SodigyDataType::String,
        }
    }

    pub fn new_bin_data(s: &Vec<u8>) -> Self {
        let mut result = Vec::with_capacity(s.len());

        for c in s.iter() {
            result.push(Rc::new(SodigyData {
                value: SodigyDataValue::SmallInt(*c as i32),
                ty: SodigyDataType::TODO,
            }));
        }

        SodigyData {
            value: SodigyDataValue::Compound(result),
            ty: SodigyDataType::TODO,
        }
    }

    pub fn try_get_func_index(&self) -> Result<usize, ()> {
        match self {
            SodigyData {
                value: SodigyDataValue::SmallInt(index),
                ty: SodigyDataType::Func,
            } => Ok(*index as usize),
            _ => Err(()),
        }
    }

    pub fn try_into_big_int(&self) -> Option<&BigInt> {
        match self {
            SodigyData {
                value: SodigyDataValue::BigInt(n),
                ty: SodigyDataType::Integer,
            } => Some(n),
            _ => None,
        }
    }

    pub fn try_into_ratio(&self) -> Option<Ratio> {
        match self {
            SodigyData {
                value: SodigyDataValue::Compound(nums),
                ty: SodigyDataType::Ratio,
            } => {
                let denom = nums[0].try_into_big_int().unwrap();
                let numer = nums[1].try_into_big_int().unwrap();

                Some(Ratio::from_denom_and_numer(denom.clone(), numer.clone()))
            },
            _ => None,
        }
    }

    pub fn is_true(&self) -> bool {
        matches!(
            self, 
            SodigyData {
                value: SodigyDataValue::SmallInt(1),
                ty: SodigyDataType::Bool,
            }
        )
    }
}

impl From<bool> for SodigyData {
    fn from(b: bool) -> SodigyData {
        SodigyData {
            value: SodigyDataValue::SmallInt(b as i32),
            ty: SodigyDataType::Bool,
        }
    }
}

#[derive(Clone)]
pub enum SodigyDataValue {
    BigInt(BigInt),
    Compound(Vec<Rc<SodigyData>>),

    // char, enum variant, and other data that are guaranteed to be small enough
    // int type values are BigInt variant regardless of their value
    SmallInt(i32),
}

// TODO: it has to be a sodigy value
#[derive(Clone)]
pub enum SodigyDataType {
    TODO,
    Char,
    Integer,
    String,
    Ratio,
    Func,
    Bool,
}
