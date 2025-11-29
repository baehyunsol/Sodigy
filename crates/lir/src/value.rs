use crate::Session;
use sodigy_number::{BigInt, InternedNumber, InternedNumberValue};
use sodigy_span::Span;
use sodigy_string::{InternedString, unintern_string};

// This is how values are represented in Sodigy runtime.
// TODO: intern compound values
#[derive(Clone, Debug)]
pub enum Value {
    Scalar(u32),
    Compound(Vec<Value>),

    // It's only used for some debug information.
    // The runtime may implement a span-renderer, or completely ignore this.
    Span(Span),
}

impl Value {
    pub fn list(elems: Vec<Value>, session: &Session) -> Value {
        let mut result = Vec::with_capacity(elems.len() + 1);
        result.push(session.number_to_value(InternedNumber::from_u32(
            elems.len() as u32,
            /* is_integer: */ true,
        )));

        result.extend(elems);
        Value::Compound(result)
    }
}

impl Session {
    pub fn number_to_value(&self, n: InternedNumber) -> Value {
        match n {
            InternedNumber { value: InternedNumberValue::SmallInt(i @ 0..), is_integer: true } => Value::Compound(vec![
                Value::Scalar(1),
                Value::Scalar(i as u32),
            ]),
            InternedNumber { value: InternedNumberValue::BigInt(BigInt { is_neg: false, nums }), is_integer: true } => {
                let mut result = vec![Value::Scalar(nums.len() as u32)];

                for n in nums.iter() {
                    result.push(Value::Scalar(*n));
                }

                Value::Compound(result)
            },
            _ => panic!("TODO: {n:?}"),
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

        Value::list(elems, self)
    }
}
