use crate::{parse_expr, AstSession, DottedNames, Expr, ExprKind, Tokens, ValueKind};
use sodigy_err::ErrorContext;
use sodigy_parse::FormattedStringElement;
use sodigy_span::SpanRange;

pub(crate) fn format_string_into_expr(
    f: &FormattedStringElement,
    span: SpanRange,
    session: &mut AstSession,
) -> Result<Expr, ()> {
    match f {
        FormattedStringElement::Literal(s) => {
            Ok(Expr {
                kind: ExprKind::Value(ValueKind::String {
                    s: session.intern_string(s.as_bytes().to_vec()),
                    is_binary: false,
                }),
                span,
            })
        },
        FormattedStringElement::Value(v) => {
            let mut v = v.to_vec();
            let mut tokens = Tokens::from_vec(&mut v);

            // it's guaranteed to exist: lexer guarantees that
            let last_span = tokens.span_end().unwrap();

            parse_expr(
                &mut tokens,
                session,
                0,
                false,
                Some(ErrorContext::ParsingFormattedString),
                last_span,
            )
        },
    }
}

pub(crate) fn merge_dotted_names(name1: &DottedNames, name2: &DottedNames) -> DottedNames {
    name1.iter().chain(name2.iter()).map(|id| *id).collect()
}
