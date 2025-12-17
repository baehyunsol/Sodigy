use super::{Pattern, PatternKind};
use sodigy_error::{Error, ErrorKind};
use sodigy_number::InternedNumberValue;
use sodigy_span::{RenderableSpan, Span};
use sodigy_token::InfixOp;

// Rules
// 1. You can do some operations[1] with 2 integers.
// 2. You can do some operations[2] with 2 numbers.
// 3. You can do some operations[1] with 2 bytes.
// 4. If one of operand is an Identifier, the other operand must be an integer/number/byte, and you can do some operations[3].
// 5. If one of operand is a DollarIdentifier, the other operand must be an integer/number/byte, and you can do some operations[2].
//
// [1]: add/sub/mul/div/rem/shl/shr/bitand/bitor/xor
// [2]: add/sub/mul/div/rem
// [3]: add/sub
#[derive(Clone, Debug)]
enum ConstPatternType {
    Ident,
    DollarIdent,
    Int(InternedNumberValue),
    Number(InternedNumberValue),
    Byte(u8),
}

impl ConstPatternType {
    pub fn render_error_singular(&self) -> &'static str {
        match self {
            ConstPatternType::Ident => "a name binding",
            ConstPatternType::DollarIdent => todo!(),  // what do I call this?
            ConstPatternType::Int(_) => "an integer",
            ConstPatternType::Number(_) => "a number",
            ConstPatternType::Byte(_) => "a byte",
        }
    }
}

// `lhs` and `rhs` are already evaluated (parser did that).
pub fn eval_const_pattern(
    op: InfixOp,
    lhs: Pattern,
    rhs: Pattern,
    op_span: Span,
) -> Result<PatternKind, Vec<Error>> {
    let (lhs_type, rhs_type) = match (
        get_const_pattern_type(&lhs),
        get_const_pattern_type(&rhs),
    ) {
        (Ok(lhs_type), Ok(rhs_type)) => (lhs_type, rhs_type),
        (Err(e1), Err(e2)) => {
            return Err(vec![e1, e2]);
        },
        (Err(e), _) | (_, Err(e)) => {
            return Err(vec![e]);
        },
    };

    match (&lhs_type, &rhs_type) {
        (ConstPatternType::Ident, ConstPatternType::Int(_) | ConstPatternType::Number(_) | ConstPatternType::Byte(_)) |
        (ConstPatternType::Int(_) | ConstPatternType::Number(_) | ConstPatternType::Byte(_), ConstPatternType::Ident) => match op {
            // nop
            InfixOp::Add | InfixOp::Sub => Ok(PatternKind::InfixOp {
                op,
                lhs: Box::new(lhs),
                rhs: Box::new(rhs),
                op_span,
            }),
            _ => Err(vec![Error {
                kind: ErrorKind::CannotEvaluateConstPattern,
                spans: vec![
                    RenderableSpan {
                        span: op_span,
                        auxiliary: false,
                        note: Some(format!("Expected `+` or `-`, got `{}`.", op.render_error())),
                    },
                    RenderableSpan {
                        span: lhs.error_span(),
                        auxiliary: true,
                        note: Some(format!("This is an lhs of the operator, which is {}.", lhs_type.render_error_singular())),
                    },
                    RenderableSpan {
                        span: rhs.error_span(),
                        auxiliary: true,
                        note: Some(format!("This is an rhs of the operator, which is {}.", rhs_type.render_error_singular())),
                    },
                ],
                note: Some(String::from("If one of the operand is a name binding, allowed operators are `+` and `-`.")),
            }]),
        },
        (ConstPatternType::Ident, _) | (_, ConstPatternType::Ident) => {
            let note = if let (ConstPatternType::Ident, ConstPatternType::Ident) = (&lhs_type, &rhs_type) {
                String::from("You cannot bind names like this.")
            } else {
                String::from("If one of the operand is a name binding, the other should be a constant.")
            };

            Err(vec![Error {
                kind: ErrorKind::CannotEvaluateConstPattern,
                spans: vec![
                    RenderableSpan {
                        span: op_span,
                        auxiliary: false,
                        note: None,
                    },
                    RenderableSpan {
                        span: lhs.error_span(),
                        auxiliary: true,
                        note: Some(format!("This is an lhs of the operator, which is {}.", lhs_type.render_error_singular())),
                    },
                    RenderableSpan {
                        span: rhs.error_span(),
                        auxiliary: true,
                        note: Some(format!("This is an rhs of the operator, which is {}.", rhs_type.render_error_singular())),
                    },
                ],
                note: Some(note),
            }])
        },
        (ConstPatternType::DollarIdent, ConstPatternType::Int(_) | ConstPatternType::Number(_) | ConstPatternType::Byte(_)) |
        (ConstPatternType::Int(_) | ConstPatternType::Number(_) | ConstPatternType::Byte(_), ConstPatternType::DollarIdent) => match op {
            // nop
            InfixOp::Add | InfixOp::Sub | InfixOp::Mul | InfixOp::Div | InfixOp::Rem |
            InfixOp::Shl | InfixOp::Shr | InfixOp::BitAnd | InfixOp::BitOr | InfixOp::Xor => Ok(PatternKind::InfixOp {
                op,
                lhs: Box::new(lhs),
                rhs: Box::new(rhs),
                op_span,
            }),
            _ => todo!(),  // err
        },
        (ConstPatternType::DollarIdent, ConstPatternType::DollarIdent) => todo!(),  // err
        (ConstPatternType::Int(lhs), ConstPatternType::Int(rhs)) => match op {
            InfixOp::Add => todo!(),
            _ => todo!(),
        },
        (ConstPatternType::Int(_), _) | (_, ConstPatternType::Int(_)) => todo!(),  // err
        (ConstPatternType::Number(lhs), ConstPatternType::Number(rhs)) => match op {
            _ => todo!(),
        },
        (ConstPatternType::Number(_), _) | (_, ConstPatternType::Number(_)) => todo!(),  // err
        (ConstPatternType::Byte(lhs), ConstPatternType::Byte(rhs)) => match op {
            _ => todo!(),
        },
    }
}

fn get_const_pattern_type(pattern: &Pattern) -> Result<ConstPatternType, Error> {
    // `a @ 1 + b @ 2` is an illegal pattern
    if let (Some(name), Some(name_span)) = (pattern.name, pattern.name_span) {
        Err(Error {
            kind: ErrorKind::CannotBindNameToConstant(name),
            spans: name_span.simple_error(),
            note: None,
        })
    }

    // `Pattern::check()` is not run yet, so we have to check this before
    // this type annotation is removed.
    else if let Some(r#type) = &pattern.r#type {
        Err(Error {
            kind: ErrorKind::CannotAnnotateType,
            spans: r#type.error_span().simple_error(),
            note: None,
        })
    }

    else {
        match &pattern.kind {
            PatternKind::Ident { .. } => Ok(ConstPatternType::Ident),
            PatternKind::DollarIdent { .. } | PatternKind::PipelineData(_) => Ok(ConstPatternType::DollarIdent),
            PatternKind::Number { n, .. } => if n.is_integer {
                Ok(ConstPatternType::Int(n.value.clone()))
            } else {
                Ok(ConstPatternType::Number(n.value.clone()))
            },
            PatternKind::Byte { b, .. } => Ok(ConstPatternType::Byte(*b)),
            _ => {
                let note = match &pattern.kind {
                    PatternKind::Wildcard(_) => String::from("Perhaps you want to bind a name?"),
                    _ => String::from("Only simple const-evaluations are implemented (TODO: document)."),
                };

                Err(Error {
                    kind: ErrorKind::CannotEvaluateConstPattern,
                    spans: pattern.error_span().simple_error(),
                    note: Some(note),
                })
            },
        }
    }
}
