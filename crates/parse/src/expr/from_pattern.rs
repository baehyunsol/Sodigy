use crate::{Expr, Pattern, PatternKind};
use sodigy_error::{Error, ErrorKind};

// These methods are used for comptime-evaluating patterns.
impl Expr {
    pub fn from_pattern(pattern: &Pattern) -> Result<Expr, Vec<Error>> {
        let mut errors = vec![];

        if let (Some(name), Some(name_span)) = (pattern.name, pattern.name_span) {
            errors.push(Error {
                kind: ErrorKind::CannotBindNameToConstant(name),
                spans: name_span.simple_error(),
                note: None,
            });
        }

        if let Some(r#type) = &pattern.r#type {
            errors.push(Error {
                kind: ErrorKind::CannotAnnotateType,
                spans: r#type.error_span_wide().simple_error(),
                note: None,
            });
        }

        let expr = match Expr::from_pattern_kind(&pattern.kind) {
            Ok(expr) => expr,
            Err(es) => {
                errors.extend(es);
                return Err(errors);
            },
        };

        if errors.is_empty() {
            Ok(expr)
        }

        else {
            Err(errors)
        }
    }

    pub fn from_pattern_kind(pattern_kind: &PatternKind) -> Result<Expr, Vec<Error>> {
        match pattern_kind {
            PatternKind::Ident { .. } => Err(vec![Error {
                kind: ErrorKind::CannotEvaluateConst,
                spans: pattern_kind.error_span_narrow().simple_error(),
                note: None,
            }]),
            PatternKind::DollarIdent { id, span } => Ok(Expr::Ident { id: *id, span: *span }),
            PatternKind::Number { n, span } => Ok(Expr::Number { n: n.clone(), span: *span }),
            PatternKind::String { binary, s, span } => Ok(Expr::String { binary: *binary, s: *s, span: *span }),
            PatternKind::InfixOp { op, lhs, rhs, op_span, .. } => match (
                Expr::from_pattern(lhs),
                Expr::from_pattern(rhs),
            ) {
                (Ok(lhs), Ok(rhs)) => Ok(Expr::InfixOp {
                    op: *op,
                    op_span: *op_span,
                    lhs: Box::new(lhs),
                    rhs: Box::new(rhs),
                }),
                (Err(e1), Err(e2)) => Err(vec![e1, e2].concat()),
                (Err(e), _) | (_, Err(e)) => Err(e),
            },
            _ => todo!(),
        }
    }
}
