use crate::Session;
use sodigy_number::{BigInt, InternedNumber};
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
            Constant::Number { n, .. } => n.into(),
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
}

impl From<&InternedNumber> for Value {
    fn from(n: &InternedNumber) -> Value {
        todo!()
    //     match n {
    //         InternedNumber {
    //             value: InternedNumberValue::SmallInt(n),
    //             is_integer: true,
    //         } => {
    //             let is_neg = *n < 0;
    //             let abs = n.abs() as u64;
    //             let nums = match abs {
    //                 0..=0xffff_ffff => vec![abs as u32],
    //                 _ => vec![(abs & 0xffff_ffff) as u32, (abs >> 32) as u32],
    //             };

    //             Value::Int(BigInt { is_neg, nums })
    //         },
    //         InternedNumber {
    //             value: InternedNumberValue::SmallInt(n),
    //             is_integer: false,
    //         } => {
    //             Value::Compound(vec![
    //                 // TODO: we have to make sure that always `numer` comes before `denom`, everywhere.
    //                 Value::Int(BigInt::from(*n as i64)),
    //                 Value::Int(BigInt::from(1i64)),
    //             ])
    //         },
    //         InternedNumber {
    //             value: n @ InternedNumberValue::SmallRatio { numer, denom },
    //             is_integer: false,
    //         } => {
    //             Value::Compound(vec![
    //                 Value::Int(BigInt::from(*numer as i64)),
    //                 Value::Int(BigInt::from(*denom as i64)),
    //             ])
    //         },
    //         InternedNumber {
    //             value: InternedNumberValue::BigInt(n),
    //             is_integer: true,
    //         } => Value::Int(n.clone()),
    //         _ => panic!("TODO: {n:?}"),
    //     }
    }
}
