use crate::Session;
use sodigy_number::{InternedNumber, InternedNumberValue};
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
    pub fn list(elems: Vec<Value>) -> Value {
        let mut result = Vec::with_capacity(elems.len() + 1);

        // TODO: push length of the list
        // result.push();

        result.extend(elems);
        Value::Compound(result)
    }
}

impl Session {
    pub fn number_to_value(&self, n: InternedNumber) -> Value {
        match n {
            InternedNumber { value: InternedNumberValue::SmallInt(i @ 0..), is_integer: true } => panic!("TODO: {n:?}"),
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

        Value::list(elems)
    }
}
