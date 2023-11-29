#![deny(unused_imports)]

//! This is very experimental. It's 99% likely to be removed (or at least heavily modified) later.

mod data;
mod hir;

pub use hir::{eval_hir, HirEvalCtxt};
use data::{SodigyData, SodigyDataType, SodigyDataValue};
use hmath::{BigInt, Ratio};
use sodigy_intern::InternedNumeric;
use std::rc::Rc;

// it has to be implemented in Sodigy when the language is complete
// it converts `42: Int` to `"42": String` (in Sodigy)
fn to_string(e: &SodigyData) -> Result<Rc<SodigyData>, ()> {
    match e {
        SodigyData {
            value: SodigyDataValue::BigInt(n),
            ty: SodigyDataType::Integer,
        } => Ok(Rc::new(SodigyData::new_string(n.to_string().as_bytes()))),
        SodigyData {
            value: SodigyDataValue::Compound(_),
            ty: SodigyDataType::Ratio,
        } => {
            let n = e.try_into_ratio().unwrap();
            Ok(Rc::new(SodigyData::new_string(n.to_string().as_bytes())))
        },
        t @ SodigyData {
            value: SodigyDataValue::Compound(elements),
            ty: SodigyDataType::List | SodigyDataType::Tuple,
        } => {
            let mut result = Vec::with_capacity(elements.len());

            for elem in elements.iter() {
                let s = to_string(elem)?;
                result.push(to_rust_string(&s)?);
            }

            let chars = result.join(&[',' as u32, ' ' as u32][..]);

            let (start, end) = if matches!(&t.ty, SodigyDataType::List) {
                ('[', ']')
            } else {
                ('(', ')')
            };

            let mut result = Vec::with_capacity(chars.len() + 2);
            result.push(start);

            for c in chars.iter() {
                result.push(char::from_u32(*c).unwrap());
            }

            result.push(end);

            Ok(Rc::new(SodigyData::new_string(result.iter().collect::<String>().as_bytes())))
        },
        SodigyData {
            value: _,
            ty: SodigyDataType::String,
        } => Ok(Rc::new(e.clone())),  // TODO: should avoid `clone`
        _ => Err(()),  // TODO
    }
}

// convert Vec<SodigyDataValue::SmallInt> to Vec<u32>
fn to_rust_string(e: &SodigyData) -> Result<Vec<u32>, ()> {
    match e {
        SodigyData {
            value: SodigyDataValue::Compound(chars),
            ty: SodigyDataType::String,
        } => {
            let mut result = Vec::with_capacity(chars.len());

            for c in chars.iter() {
                if let SodigyDataValue::SmallInt(c) = &c.value {
                    result.push(*c as u32);
                }

                else {
                    return Err(());
                }
            }

            Ok(result)
        },
        _ => Err(()),
    }
}

pub enum ConvertError {
    NotInt,
    NotRatio,
    TODO(String),
}

pub trait IntoHmath {
    fn into_hmath_big_int(&self) -> Result<BigInt, ConvertError>;
    fn into_hmath_ratio(&self) -> Result<Ratio, ConvertError>;
}

impl IntoHmath for InternedNumeric {
    fn into_hmath_big_int(&self) -> Result<BigInt, ConvertError> {
        if let Some(n) = self.try_unwrap_small_int() {
            Ok(BigInt::from(n))
        }

        else if let Some((digits, exp)) = self.try_unwrap_digits_and_exp_from_int() {
            let res = format!(
                "{}e{exp}",
                String::from_utf8(digits).unwrap(),
            ).parse::<Ratio>().unwrap();

            Ok(res.truncate_bi())
        }

        else {
            Err(ConvertError::NotInt)
        }
    }

    fn into_hmath_ratio(&self) -> Result<Ratio, ConvertError> {
        if let Some((digits, exp)) = self.try_unwrap_digits_and_exp_from_ratio() {
            Ok(format!(
                "{}e{exp}",
                String::from_utf8(digits).unwrap(),
            ).parse::<Ratio>().unwrap())
        }

        else {
            Err(ConvertError::NotRatio)
        }
    }
}
