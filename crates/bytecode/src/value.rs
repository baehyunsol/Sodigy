use crate::{ExprHash, Session};
use sodigy_number::{BigInt, Ratio, unintern_number};
use sodigy_span::Span;
use sodigy_string::unintern_string;
use sodigy_token::Constant;
use std::collections::hash_map::Entry;

// This is how values are represented in Sodigy runtime.
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

#[derive(Clone, Debug)]
pub enum InternedValue {
    Interned(ExprHash),
    Scalar(u32),
}

impl Session<'_, '_> {
    // FIXME: so many unwraps!
    pub fn lower_constant(&mut self, constant: &Constant) -> InternedValue {
        match constant {
            Constant::Number { n, .. } => match self.number_to_expr_hash.entry(*n) {
                Entry::Occupied(e) => InternedValue::Interned(*e.get()),
                Entry::Vacant(e) => {
                    let is_integer = n.is_integer();
                    let n = unintern_number(*n, &self.intermediate_dir).unwrap();
                    let value = if is_integer {
                        Value::Int(n.numer)
                    } else {
                        let Ratio { numer, denom } = n;
                        // TODO: we have to make sure that always `numer` comes before `denom`, everywhere.
                        Value::Compound(vec![Value::Int(numer), Value::Int(denom)])
                    };
                    let expr_hash = ExprHash::from_const(&value);

                    e.insert(expr_hash);
                    self.data_section.insert(expr_hash, value);
                    InternedValue::Interned(expr_hash)
                },
            },
            Constant::String { s, binary, .. } => match self.string_to_expr_hash.entry((*s, *binary)) {
                Entry::Occupied(e) => InternedValue::Interned(*e.get()),
                Entry::Vacant(e) => {
                    let b = unintern_string(*s, &self.intermediate_dir).unwrap().unwrap();
                    let elems: Vec<Value> = if *binary {
                        b.iter().map(
                            |b| Value::Scalar(*b as u32)
                        ).collect()
                    } else {
                        String::from_utf8(b).unwrap().chars().map(
                            |c| Value::Scalar(c as u32)
                        ).collect()
                    };
                    let value = Value::List(elems);
                    let expr_hash = ExprHash::from_const(&value);

                    e.insert(expr_hash);
                    self.data_section.insert(expr_hash, value);
                    InternedValue::Interned(expr_hash)
                },
            },
            Constant::Char { ch, .. } => InternedValue::Scalar(*ch),
            Constant::Byte { b, .. } => InternedValue::Scalar(*b as u32),
            Constant::Scalar(n) => InternedValue::Scalar(*n),
        }
    }

    pub fn intern_value(&mut self, v: &Value) -> InternedValue {
        match v {
            Value::Scalar(n) => InternedValue::Scalar(*n),
            _ => {
                let expr_hash = ExprHash::from_const(v);
                self.data_section.insert(expr_hash, v.clone());
                InternedValue::Interned(expr_hash)
            },
        }
    }
}
