use sodigy_error::{Error, ErrorKind};
use sodigy_number::{
    BigInt,
    InternedNumber,
    add_ratio,
    div_ratio,
    intern_big_int,
    intern_ratio,
    mul_ratio,
    shl_ubi,
    shr_ubi,
    sub_ratio,
    unintern_big_int,
    unintern_number,
};
use sodigy_span::{RenderableSpan, Span};
use sodigy_token::{InfixOp, PrefixOp};

pub fn eval_number_prefix_op(
    op: PrefixOp,
    op_span: Span,
    rhs: &InternedNumber,
    rhs_span: &Span,
    intermediate_dir: &str,
) -> Result<InternedNumber, Vec<Error>> {
    match op {
        PrefixOp::Neg => Ok(rhs.negate(intermediate_dir).unwrap()),
        PrefixOp::Not => Err(vec![Error {
            kind: ErrorKind::CannotEvaluateConst,
            spans: op_span.simple_error(),
            note: Some(String::from("Const-eval is not implemented for `!` operator.")),
        }]),
        PrefixOp::Range { inclusive } => Err(vec![Error {
            kind: ErrorKind::CannotEvaluateConst,
            spans: op_span.simple_error(),
            note: Some(format!(
                "Const-eval is not implemented for `{}` operator.",
                if inclusive { "..=" } else { ".." },
            )),
        }]),
    }
}

// FIXME: So many unwraps...
pub fn eval_number_infix_op(
    op: InfixOp,
    op_span: &Span,
    lhs: InternedNumber,
    lhs_span: &Span,
    rhs: InternedNumber,
    rhs_span: &Span,
    intermediate_dir: &str,
) -> Result<InternedNumber, Vec<Error>> {
    match op {
        InfixOp::Add |
        InfixOp::Sub |
        InfixOp::Mul |
        InfixOp::Div => {
            let lhs_ratio = unintern_number(lhs, intermediate_dir).unwrap();
            let rhs_ratio = unintern_number(rhs, intermediate_dir).unwrap();

            match (op, lhs.is_integer(), rhs.is_integer()) {
                (InfixOp::Add, true, true) => Ok(intern_ratio(&add_ratio(&lhs_ratio, &rhs_ratio), true, intermediate_dir).unwrap()),
                (InfixOp::Add, false, false) => Ok(intern_ratio(&add_ratio(&lhs_ratio, &rhs_ratio), false, intermediate_dir).unwrap()),
                (InfixOp::Add, _, _) => Err(vec![infix_op_type_error(op, op_span, lhs, rhs)]),
                (InfixOp::Sub, true, true) => Ok(intern_ratio(&sub_ratio(&lhs_ratio, &rhs_ratio), true, intermediate_dir).unwrap()),
                (InfixOp::Sub, false, false) => Ok(intern_ratio(&sub_ratio(&lhs_ratio, &rhs_ratio), false, intermediate_dir).unwrap()),
                (InfixOp::Sub, _, _) => Err(vec![infix_op_type_error(op, op_span, lhs, rhs)]),
                (InfixOp::Mul, true, true) => Ok(intern_ratio(&mul_ratio(&lhs_ratio, &rhs_ratio), true, intermediate_dir).unwrap()),
                (InfixOp::Mul, false, false) => Ok(intern_ratio(&mul_ratio(&lhs_ratio, &rhs_ratio), false, intermediate_dir).unwrap()),
                (InfixOp::Mul, _, _) => Err(vec![infix_op_type_error(op, op_span, lhs, rhs)]),
                (InfixOp::Div, true, true) => Ok(intern_ratio(&div_ratio(&lhs_ratio, &rhs_ratio), true, intermediate_dir).unwrap()),
                (InfixOp::Div, false, false) => Ok(intern_ratio(&div_ratio(&lhs_ratio, &rhs_ratio), false, intermediate_dir).unwrap()),
                (InfixOp::Div, _, _) => Err(vec![infix_op_type_error(op, op_span, lhs, rhs)]),
                _ => unreachable!(),
            }
        },
        InfixOp::Shl | InfixOp::Shr => {
            if lhs.is_integer() && rhs.is_integer() {
                let lhs_int = unintern_big_int(lhs, intermediate_dir).unwrap();
                let rhs_int = unintern_big_int(rhs, intermediate_dir).unwrap();
                let rhs_int = match i128::try_from(&rhs_int) {
                    Ok(n @ 0..4294967296) => n as u32,
                    _ if rhs_int.is_neg => todo!(),
                    _ => todo!(),
                };

                let nums = match op {
                    InfixOp::Shl => shl_ubi(&lhs_int.nums, rhs_int),
                    InfixOp::Shr => shr_ubi(&lhs_int.nums, rhs_int),
                    _ => unreachable!(),
                };

                Ok(intern_big_int(
                    &BigInt {
                        nums,
                        is_neg: lhs_int.is_neg,
                    },
                    true,
                    intermediate_dir,
                ).unwrap())
            }

            else {
                let mut spans = vec![RenderableSpan {
                    span: op_span.clone(),
                    auxiliary: false,
                    note: None,
                }];

                if !lhs.is_integer() {
                    spans.push(RenderableSpan {
                        span: lhs_span.clone(),
                        auxiliary: true,
                        note: Some(String::from("This is not an integer.")),
                    });
                }

                if !rhs.is_integer() {
                    spans.push(RenderableSpan {
                        span: rhs_span.clone(),
                        auxiliary: true,
                        note: Some(String::from("This is not an integer.")),
                    });
                }

                match op {
                    InfixOp::Shl => Err(vec![Error {
                        kind: ErrorKind::CannotEvaluateConst,
                        spans,
                        note: Some(String::from("Lhs and rhs of the `>>` operator has to be an integer.")),
                    }]),
                    InfixOp::Shr => Err(vec![Error {
                        kind: ErrorKind::CannotEvaluateConst,
                        spans,
                        note: Some(String::from("Lhs and rhs of the `<<` operator has to be an integer.")),
                    }]),
                    _ => unreachable!(),
                }
            }
        },
        _ => Err(vec![Error::todo(89470, "more const eval", op_span.clone())]),
    }
}

fn infix_op_type_error(
    op: InfixOp,
    op_span: &Span,
    lhs: InternedNumber,
    rhs: InternedNumber,
) -> Error {
    Error {
        kind: ErrorKind::CannotEvaluateConst,
        spans: op_span.simple_error(),
        note: Some(format!(
            "`{}` operator is not implemented for `{}` and `{}`.",
            op.render_error(),
            if lhs.is_integer() { "Int" } else { "Number" },
            if rhs.is_integer() { "Int" } else { "Number" },
        )),
    }
}
