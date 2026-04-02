use sodigy_error::{Error, ErrorKind};
use sodigy_number::{
    InternedNumber,
    add_ratio,
    div_ratio,
    intern_ratio,
    mul_ratio,
    sub_ratio,
    unintern_number,
};
use sodigy_span::Span;
use sodigy_token::{InfixOp, PrefixOp};

pub fn eval_number_prefix_op(
    op: PrefixOp,
    op_span: Span,
    rhs: &InternedNumber,
) -> Result<InternedNumber, Vec<Error>> {
    match op {
        PrefixOp::Neg => Ok(rhs.negate()),
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
    op_span: Span,
    lhs: &InternedNumber,
    rhs: &InternedNumber,
    intermediate_dir: &str,
) -> Result<InternedNumber, Vec<Error>> {
    if lhs.is_integer() != rhs.is_integer() {
        return Err(vec![Error {
            kind: ErrorKind::CannotEvaluateConst,
            spans: op_span.simple_error(),
            note: Some(format!(
                "Lhs is {} while rhs is {}.",
                if lhs.is_integer() { "an integer" } else { "a number" },
                if rhs.is_integer() { "an integer" } else { "a number" },
            )),
        }]);
    }

    let is_integer = lhs.is_integer();
    let lhs_ratio = unintern_number(*lhs, intermediate_dir).unwrap();
    let rhs_ratio = unintern_number(*rhs, intermediate_dir).unwrap();

    match op {
        InfixOp::Add => Ok(intern_ratio(&add_ratio(&lhs_ratio, &rhs_ratio), is_integer, intermediate_dir).unwrap()),
        InfixOp::Sub => Ok(intern_ratio(&sub_ratio(&lhs_ratio, &rhs_ratio), is_integer, intermediate_dir).unwrap()),
        InfixOp::Mul => Ok(intern_ratio(&mul_ratio(&lhs_ratio, &rhs_ratio), is_integer, intermediate_dir).unwrap()),
        InfixOp::Div => {
            let value = intern_ratio(&div_ratio(&lhs_ratio, &rhs_ratio), is_integer, intermediate_dir).unwrap();

            if lhs.is_integer() {
                // We have to truncate the result!
                todo!()
            }

            else {
                Ok(value)
            }
        },
        _ => Err(vec![Error::todo(89470, "more const eval", op_span)]),
    }
}
