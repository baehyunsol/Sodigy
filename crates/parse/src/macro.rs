use crate::{Expr, Tokens, Type};
use sodigy_error::{Error, ErrorKind, ErrorToken};
use sodigy_span::{RenderableSpan, Span};
use sodigy_string::InternedString;
use sodigy_token::{Token, TokenKind};

#[derive(Clone, Debug)]
pub enum MacroKind {
    IncludeString {
        path: InternedString,
    },
    IncludeBytes {
        path: InternedString,
    },
    TypeName {
        r#type: Type,
    },
    TypeNameOfValue {
        value: Expr,
    },
    NumberOfVariants {
        r#type: Type,
    },
    NumberOfFields {
        r#type: Type,
    },
    NameOfVariants {
        r#type: Type,
    },
    NameOfFields {
        r#type: Type,
    },
    File,
    ModulePath,
    Line,
    Column,
}

pub fn try_parse_macro(
    id: InternedString,
    macro_span: Span,
    group_span: Span,
    group_tokens: &[Token],
    intermediate_dir: &String,
) -> Result<Expr, Vec<Error>> {
    let tokens = Tokens::new(
        group_tokens,
        group_span.end(),
        false,
        intermediate_dir,
    );

    match id {
        _ if id.eq(b"include_string") => todo!(),
        _ if id.eq(b"include_bytes") => todo!(),
        _ if id.eq(b"type_name") => Ok(Expr::Macro {
            kind: Box::new(MacroKind::TypeName { r#type: parse_type(tokens)? }),
            macro_span,
            group_span,
        }),
        _ if id.eq(b"type_name_of_value") => Ok(Expr::Macro {
            kind: Box::new(MacroKind::TypeNameOfValue { value: parse_expr(tokens)? }),
            macro_span,
            group_span,
        }),
        _ if id.eq(b"number_of_variants") => Ok(Expr::Macro {
            kind: Box::new(MacroKind::NumberOfVariants { r#type: parse_type(tokens)? }),
            macro_span,
            group_span,
        }),
        _ if id.eq(b"number_of_fields") => Ok(Expr::Macro {
            kind: Box::new(MacroKind::NumberOfFields { r#type: parse_type(tokens)? }),
            macro_span,
            group_span,
        }),
        _ if id.eq(b"name_of_variants") => Ok(Expr::Macro {
            kind: Box::new(MacroKind::NameOfVariants { r#type: parse_type(tokens)? }),
            macro_span,
            group_span,
        }),
        _ if id.eq(b"name_of_fields") => Ok(Expr::Macro {
            kind: Box::new(MacroKind::NameOfFields { r#type: parse_type(tokens)? }),
            macro_span,
            group_span,
        }),
        _ if id.eq(b"file") => {
            tokens.empty_or_error()?;
            Ok(Expr::Macro {
                kind: Box::new(MacroKind::File),
                macro_span,
                group_span,
            })
        },
        _ if id.eq(b"module_path") => {
            tokens.empty_or_error()?;
            Ok(Expr::Macro {
                kind: Box::new(MacroKind::ModulePath),
                macro_span,
                group_span,
            })
        },
        _ if id.eq(b"line") => {
            tokens.empty_or_error()?;
            Ok(Expr::Macro {
                kind: Box::new(MacroKind::Line),
                macro_span,
                group_span,
            })
        },
        _ if id.eq(b"column") => {
            tokens.empty_or_error()?;
            Ok(Expr::Macro {
                kind: Box::new(MacroKind::Column),
                macro_span,
                group_span,
            })
        },
        _ => Err(vec![Error {
            kind: ErrorKind::UndefinedMacro(id),
            spans: macro_span.simple_error(),
            note: None,
        }]),
    }
}

fn parse_expr(mut tokens: Tokens) -> Result<Expr, Vec<Error>> {
    let expr = tokens.parse_expr(true)?;
    tokens.empty_or_error()?;
    Ok(expr)
}

fn parse_type(mut tokens: Tokens) -> Result<Type, Vec<Error>> {
    let r#type = tokens.parse_type()?;
    tokens.empty_or_error()?;
    Ok(r#type)
}

fn parse_string(tokens: Tokens) -> Result<InternedString, Vec<Error>> {
    match tokens.peek2() {
        (Some(Token { kind: TokenKind::String { binary: false, raw: false, regex: false, s }, .. }), None) => Ok(*s),
        (Some(Token { kind: TokenKind::String { .. }, span }), None) => Err(vec![Error {
            kind: ErrorKind::UnexpectedToken {
                expected: ErrorToken::String,
                got: ErrorToken::Expr,
            },
            spans: vec![
                RenderableSpan {
                    span: span.start(),
                    auxiliary: false,
                    note: Some(String::from("Remove this prefix")),
                },
            ],
            note: None,
        }]),
        (Some(Token { kind: TokenKind::String { .. }, .. }), Some(t)) => Err(vec![Error {
            kind: ErrorKind::UnexpectedToken {
                expected: ErrorToken::Nothing,
                got: (&t.kind).into(),
            },
            spans: t.span.simple_error(),
            note: None,
        }]),
        (Some(t), _) => Err(vec![Error {
            kind: ErrorKind::UnexpectedToken {
                expected: ErrorToken::String,
                got: (&t.kind).into(),
            },
            spans: t.span.simple_error(),
            note: None,
        }]),
        (None, _) => Err(vec![tokens.unexpected_end(ErrorToken::String)]),
    }
}
