use crate::{error::AstError, parse_expr, AstSession, DottedNames, Expr, ExprKind, Tokens, ValueKind};
use sodigy_error::ErrorContext;
use sodigy_parse::FormattedStringElement;
use sodigy_session::SodigySession;
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
                    content: session.intern_string(s.as_bytes().to_vec()),
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

pub(crate) fn try_into_char(s: &[u8]) -> Result<char, IntoCharErr> {
    // let's not call `.to_vec` for lengthy strings
    if s.len() > 4 {
        return Err(IntoCharErr::TooLong);
    }

    let s = String::from_utf8(s.to_vec());

    if let Err(_) = s {
        return Err(IntoCharErr::InvalidUtf8);
    }

    let s = s.unwrap();
    let mut chars = s.chars();

    match chars.next() {
        Some(c) => {
            let c = c;

            if chars.next().is_some() {
                return Err(IntoCharErr::TooLong);
            }

            Ok(c)
        },
        None => Err(IntoCharErr::EmptyString),
    }
}

pub(crate) enum IntoCharErr {
    TooLong,
    InvalidUtf8,
    EmptyString,
}

impl IntoCharErr {
    pub fn into_ast_error(&self, span: SpanRange) -> AstError {
        match self {
            IntoCharErr::TooLong => AstError::too_long_char_literal(span),
            IntoCharErr::InvalidUtf8 => AstError::invalid_utf8(span),
            IntoCharErr::EmptyString => AstError::empty_char_literal(span),
        }
    }
}
