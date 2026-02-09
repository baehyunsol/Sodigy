use crate::Session;
use sodigy_number::{InternedNumber, InternedNumberValue};
use sodigy_span::Span;
use sodigy_string::{InternedString, unintern_string};
use sodigy_token::Constant;

// This is how values are represented in Sodigy runtime.
// TODO: intern compound values
#[derive(Clone, Debug)]
pub enum Value {
    Scalar(u32),
    Compound(Vec<Value>),
    FuncPointer {
        def_span: Span,
        program_counter: Option<usize>,
    },

    // It's only used for some debug information.
    // The runtime may implement a span-renderer, or completely ignore this.
    Span(Span),
}

impl Value {
    pub fn list(elems: Vec<Value>) -> Value {
        let mut result = Vec::with_capacity(elems.len() + 1);
        result.push((&InternedNumber::from_u32(
            elems.len() as u32,
            /* is_integer: */ true,
        )).into());

        result.extend(elems);
        Value::Compound(result)
    }
}

impl Session {
    pub fn lower_constant(&self, constant: &Constant) -> Value {
        match constant {
            Constant::Number { n, .. } => n.into(),
            Constant::String { s, binary, .. } => self.string_to_value(*s, *binary),
            Constant::Char { ch, .. } => Value::Scalar(*ch as u32),
            Constant::Byte { b, .. } => Value::Scalar(*b as u32),
        }
    }

    // TODO: we need some kinda intern mechanism here... again!
    // FIXME: so many unwraps!
    pub fn string_to_value(&self, s: InternedString, binary: bool) -> Value {
        let b = unintern_string(s, &self.intermediate_dir).unwrap().unwrap();
        let elems: Vec<Value> = if binary {
            b.iter().map(
                |b| Value::Scalar(*b as u32)
            ).collect()
        } else {
            String::from_utf8(b).unwrap().chars().map(
                |c| Value::Scalar(c as u32)
            ).collect()
        };

        Value::list(elems)
    }
}

impl From<&InternedNumber> for Value {
    fn from(n: &InternedNumber) -> Value {
        match n {
            InternedNumber {
                value: InternedNumberValue::SmallInt(n),
                is_integer: true,
            } => {
                let is_neg = *n < 0;
                let abs = n.abs() as u64;

                match abs {
                    0..=0xffff_ffff => Value::Compound(vec![
                        Value::Scalar(if is_neg { 0x8000_0001 } else { 1 }),
                        Value::Scalar(abs as u32),
                    ]),
                    _ => Value::Compound(vec![
                        Value::Scalar(if is_neg { 0x8000_0002 } else { 2 }),
                        Value::Scalar((abs & 0xffff_ffff) as u32),
                        Value::Scalar((abs >> 32) as u32),
                    ]),
                }
            },
            InternedNumber {
                value: InternedNumberValue::BigInt(n),
                is_integer: true,
            } => {
                let mut value = vec![
                    Value::Scalar(if n.is_neg { 0x8000_0000 | n.nums.len() as u32 } else { n.nums.len() as u32 }),
                ];
                value.extend(n.nums.iter().map(
                    |n| Value::Scalar(*n)
                ).collect::<Vec<_>>());

                Value::Compound(value)
            },
            _ => todo!(),
        }
    }
}
