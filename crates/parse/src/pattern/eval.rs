use super::{Pattern, PatternKind};
use sodigy_error::{Error, ErrorKind};
use sodigy_number::InternedNumberValue;
use sodigy_span::Span;
use sodigy_token::InfixOp;

// Rules
// 1. You can do some operations[1] with 2 integers.
// 2. You can do some operations[2] with 2 numbers.
// 3. You can do some operations[1] with 2 bytes.
// 4. If one of operand is an Identifier, the other operand must be an integer/number/byte, and the operator must be add/sub.
// 5. If one of operand is a DollarIdentifier, the other operand must be an integer/number/byte, and you can do some operations[2].
//
// [1]: add/sub/mul/div/rem/shl/shr/bitand/bitor/xor
// [2]: add/sub/mul/div/rem
enum ConstPatternType {
    Identifier,
    DollarIdentifier,
    Int(InternedNumberValue),
    Number(InternedNumberValue),
    Byte(u8),
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

    match (lhs_type, rhs_type) {
        (ConstPatternType::Identifier, ConstPatternType::Int(_) | ConstPatternType::Number(_) | ConstPatternType::Byte(_)) |
        (ConstPatternType::Int(_) | ConstPatternType::Number(_) | ConstPatternType::Byte(_), ConstPatternType::Identifier) => match op {
            // nop
            InfixOp::Add | InfixOp::Sub => Ok(PatternKind::InfixOp {
                op,
                lhs: Box::new(lhs),
                rhs: Box::new(rhs),
                op_span,
            }),
            _ => todo!(),  // err
        },
        (ConstPatternType::Identifier, _) | (_, ConstPatternType::Identifier) => todo!(),  // err
        (ConstPatternType::DollarIdentifier, ConstPatternType::Int(_) | ConstPatternType::Number(_) | ConstPatternType::Byte(_)) |
        (ConstPatternType::Int(_) | ConstPatternType::Number(_) | ConstPatternType::Byte(_), ConstPatternType::DollarIdentifier) => match op {
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
        (ConstPatternType::DollarIdentifier, _) | (_, ConstPatternType::DollarIdentifier) => todo!(),  // err
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
        Err(todo!())
    }

    // `Pattern::check()` is not run yet, so we have to check this before
    // this type annotation is removed.
    else if let Some(r#type) = &pattern.r#type {
        Err(todo!())
    }

    else {
        match &pattern.kind {
            PatternKind::Identifier { .. } => Ok(ConstPatternType::Identifier),
            PatternKind::DollarIdentifier { .. } => Ok(ConstPatternType::DollarIdentifier),
            PatternKind::Number { n, .. } => if n.is_integer {
                Ok(ConstPatternType::Int(n.value.clone()))
            } else {
                Ok(ConstPatternType::Number(n.value.clone()))
            },
            PatternKind::Byte { b, .. } => Ok(ConstPatternType::Byte(*b)),
            PatternKind::InfixOp { .. } => Err(todo!()),  // I want more detailed error message here
            PatternKind::Wildcard { .. } => Err(todo!()),  // again, I want a better error message for this
            _ => Err(todo!()),  // cannot const eval
        }
    }
}
