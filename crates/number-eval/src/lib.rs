use sodigy_error::{Error, ErrorKind};
use sodigy_number::{
    InternedNumber,
    add_ratio,
    div_ratio,
    intern_number,
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
        PrefixOp::Neg => {
            let mut n = rhs.clone();
            n.negate_mut();
            Ok(n)
        },
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

pub fn eval_number_infix_op(
    op: InfixOp,
    op_span: Span,
    lhs: &InternedNumber,
    rhs: &InternedNumber,
) -> Result<InternedNumber, Vec<Error>> {
    if lhs.is_integer != rhs.is_integer {
        return Err(vec![Error {
            kind: ErrorKind::CannotEvaluateConst,
            spans: op_span.simple_error(),
            note: Some(format!(
                "Lhs is {} while rhs is {}.",
                if lhs.is_integer { "an integer" } else { "a number" },
                if rhs.is_integer { "an integer" } else { "a number" },
            )),
        }]);
    }

    let lhs_ratio = unintern_number(lhs.value.clone());
    let rhs_ratio = unintern_number(rhs.value.clone());

    match op {
        InfixOp::Add => {
            let r = add_ratio(&lhs_ratio, &rhs_ratio);
            let value = intern_number(r);
            Ok(InternedNumber { value, is_integer: lhs.is_integer })
        },
        InfixOp::Sub => {
            let r = sub_ratio(&lhs_ratio, &rhs_ratio);
            let value = intern_number(r);
            Ok(InternedNumber { value, is_integer: lhs.is_integer })
        },
        InfixOp::Mul => {
            let r = mul_ratio(&lhs_ratio, &rhs_ratio);
            let value = intern_number(r);
            Ok(InternedNumber { value, is_integer: lhs.is_integer })
        },
        InfixOp::Div => {
            let r = div_ratio(&lhs_ratio, &rhs_ratio);
            let value = intern_number(r);

            if lhs.is_integer {
                // We have to truncate the result!
                todo!()
            }

            else {
                Ok(InternedNumber { value, is_integer: lhs.is_integer })
            }
        },
        _ => Err(vec![Error::todo(89470, "more const eval", op_span)]),
    }
}
