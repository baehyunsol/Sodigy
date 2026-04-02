use crate::Session;
use sodigy_number::{BigInt, InternedNumber, Ratio, unintern_number};
use sodigy_span::Span;
use sodigy_string::{InternedString, unintern_string};
use sodigy_token::Constant;

// This is how values are represented in Sodigy runtime.
// TODO: intern values
#[derive(Clone, Debug)]
pub enum Value {
    Scalar(u32),
    Int(BigInt),

    // List types are converted to `Value::List`. It's runtime's choice to
    // treat `Value::List` and `Value::Compound` differently or not.
    List(Vec<Value>),
    Compound(Vec<Value>),

    FuncPointer {
        def_span: Span,
        program_counter: Option<usize>,
    },

    // It's only used for some debug information.
    // The runtime may implement a span-renderer, or completely ignore this.
    Span(Span),
}

impl Session<'_, '_> {
    pub fn lower_constant(&self, constant: &Constant) -> Value {
        match constant {
            Constant::Number { n, .. } => self.number_to_value(*n),
            Constant::String { s, binary, .. } => self.string_to_value(*s, *binary),
            Constant::Char { ch, .. } => Value::Scalar(*ch),
            Constant::Byte { b, .. } => Value::Scalar(*b as u32),
            Constant::Scalar(n) => Value::Scalar(*n),
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

        Value::List(elems)
    }

    pub fn number_to_value(&self, n: InternedNumber) -> Value {
        let is_integer = n.is_integer();
        let n = unintern_number(n, &self.intermediate_dir).unwrap();

        if is_integer {
            Value::Int(n.numer)
        }

        else {
            let Ratio { numer, denom } = n;
            // TODO: we have to make sure that always `numer` comes before `denom`, everywhere.
            Value::Compound(vec![Value::Int(numer), Value::Int(denom)])
        }
    }
}
