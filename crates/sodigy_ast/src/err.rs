use crate::{IdentWithSpan, Token, TokenKind};
use sodigy_err::{ErrorContext, ExtraErrInfo, SodigyError, SodigyErrorKind};
use sodigy_intern::{InternedString, InternSession};
use sodigy_keyword::Keyword;
use sodigy_parse::{Delim, Punct};
use sodigy_span::SpanRange;

mod fmt;

#[derive(Clone)]
pub struct AstError {
    pub(crate) kind: AstErrorKind,
    spans: Vec<SpanRange>,
    extra: ExtraErrInfo,
}

impl AstError {
    pub fn unexpected_token(token: Token, expected_token: ExpectedToken) -> Self {
        let mut extra = ExtraErrInfo::none();

        match token.kind {
            TokenKind::Keyword(k) => {
                // expected ident, but got keyword K
                // tell them that K is not an identifier
                match expected_token {
                    ExpectedToken::AnyIdentifier
                    | ExpectedToken::AnyExpression
                    | ExpectedToken::IdentOrBrace => {
                        extra.set_message(format!(
                            "`{k}` is a keyword, not an identifier. If you want to use `{k}` as an identifier, try `{k}_`"
                        ));
                    },
                    _ => {},
                }
            },
            TokenKind::Identifier(id) => {
                match expected_token {
                    ExpectedToken::AnyStatement => {
                        // TODO: if it seems like that `id` is a typo, tell them
                    },
                    _ => {},
                }
            },
            _ => {},
        }

        AstError {
            kind: AstErrorKind::UnexpectedToken(token.kind, expected_token),
            spans: vec![token.span],
            extra,
        }
    }

    pub fn unexpected_end(span: SpanRange, expected_token: ExpectedToken) -> Self {
        AstError {
            kind: AstErrorKind::UnexpectedEnd(expected_token),
            spans: vec![span],
            extra: ExtraErrInfo::none(),
        }
    }

    pub fn empty_generic_list(span: SpanRange) -> Self {
        AstError {
            kind: AstErrorKind::EmptyGenericList,
            spans: vec![span],
            extra: ExtraErrInfo::none(),
        }
    }

    pub fn binary_char(span: SpanRange) -> Self {
        AstError {
            kind: AstErrorKind::BinaryChar,
            spans: vec![span],
            extra: ExtraErrInfo::none(),
        }
    }

    pub fn empty_char_literal(span: SpanRange) -> Self {
        AstError {
            kind: AstErrorKind::EmptyCharLiteral,
            spans: vec![span],
            extra: ExtraErrInfo::none(),
        }
    }

    pub fn too_long_char_literal(span: SpanRange) -> Self {
        AstError {
            kind: AstErrorKind::TooLongCharLiteral,
            spans: vec![span],
            extra: ExtraErrInfo::none(),
        }
    }

    pub fn empty_scope_block(span: SpanRange) -> Self {
        AstError {
            kind: AstErrorKind::EmptyScopeBlock,
            spans: vec![span],
            extra: ExtraErrInfo::none(),
        }
    }

    pub fn empty_match_body(span: SpanRange) -> Self {
        AstError {
            kind: AstErrorKind::EmptyMatchBody,
            spans: vec![span],
            extra: ExtraErrInfo::at_context(ErrorContext::ParsingMatchBody),
        }
    }

    pub fn func_arg_without_type(func_name: InternedString, arg: IdentWithSpan) -> Self {
        AstError {
            kind: AstErrorKind::FuncArgWithoutType { arg_name: *arg.id(), func_name },
            spans: vec![*arg.span()],
            extra: ExtraErrInfo::at_context(ErrorContext::ParsingFuncArgs),
        }
    }

    pub fn todo(msg: &str, span: SpanRange) -> Self {
        AstError {
            kind: AstErrorKind::TODO(msg.to_string()),
            spans: vec![span],
            extra: ExtraErrInfo::none(),
        }
    }
}

impl SodigyError<AstErrorKind> for AstError {
    fn get_mut_error_info(&mut self) -> &mut ExtraErrInfo {
        &mut self.extra
    }

    fn get_error_info(&self) -> &ExtraErrInfo {
        &self.extra
    }

    fn get_first_span(&self) -> SpanRange {
        self.spans[0]
    }

    fn get_spans(&self) -> &[SpanRange] {
        &self.spans
    }

    fn err_kind(&self) -> &AstErrorKind {
        &self.kind
    }
}

#[derive(Clone)]
pub enum AstErrorKind {
    UnexpectedToken(TokenKind, ExpectedToken),
    UnexpectedEnd(ExpectedToken),
    EmptyGenericList,
    BinaryChar,
    EmptyCharLiteral,
    TooLongCharLiteral,
    EmptyScopeBlock,
    EmptyMatchBody,
    FuncArgWithoutType { arg_name: InternedString, func_name: InternedString },
    TODO(String),
}

impl SodigyErrorKind for AstErrorKind {
    fn msg(&self, _: &mut InternSession) -> String {
        match self {
            AstErrorKind::UnexpectedToken(token, expected) => format!("expected {expected}, got `{token}`"),
            AstErrorKind::UnexpectedEnd(expected) => format!("expected {expected}, got nothing"),
            AstErrorKind::EmptyGenericList => String::from("empty generic parameter list"),
            AstErrorKind::BinaryChar => String::from("binary character literal"),
            AstErrorKind::EmptyCharLiteral => String::from("empty character literal"),
            AstErrorKind::TooLongCharLiteral => String::from("too long character literal"),
            AstErrorKind::EmptyScopeBlock => String::from("expected an expression or local values, got nothing"),
            AstErrorKind::EmptyMatchBody => String::from("expected a pattern, got nothing"),
            AstErrorKind::FuncArgWithoutType { .. } => String::from("a function argument without a type annotation"),
            AstErrorKind::TODO(s) => format!("not implemented: {s}"),
        }
    }

    fn help(&self, _: &mut InternSession) -> String {
        match self {
            AstErrorKind::UnexpectedToken(_, _)
            | AstErrorKind::UnexpectedEnd(_)
            | AstErrorKind::EmptyScopeBlock
            | AstErrorKind::EmptyMatchBody
            | AstErrorKind::TODO(_) => String::new(),
            AstErrorKind::EmptyGenericList => String::from("Try remove angle brackets."),
            AstErrorKind::BinaryChar => String::from("Try remove prefix `b`."),
            AstErrorKind::FuncArgWithoutType { arg_name, func_name } => format!(
                "Argument `{arg_name}` of `{func_name}` needs a type annotation."
            ),
            AstErrorKind::EmptyCharLiteral
            | AstErrorKind::TooLongCharLiteral => String::from("If you meant to write a string literal, use double quotes."),
        }
    }
}

#[derive(Clone)]
pub enum ExpectedToken {
    AnyIdentifier,
    AnyExpression,
    AnyStatement,
    AnyDocComment,
    IdentOrBrace,
    Nothing,

    /// things that can follow an expression
    PostExpr,

    /// func call, not func def
    FuncArgs,
    Specific(Vec<TokenKind>),
}

impl ExpectedToken {
    pub fn specific(t: TokenKind) -> Self {
        ExpectedToken::Specific(vec![t])
    }

    pub fn ident() -> Self {
        ExpectedToken::AnyIdentifier
    }

    pub fn expr() -> Self {
        ExpectedToken::AnyExpression
    }

    pub fn stmt() -> Self {
        ExpectedToken::AnyStatement
    }

    pub fn nothing() -> Self {
        ExpectedToken::Nothing
    }

    pub fn post() -> Self {
        ExpectedToken::PostExpr
    }

    /// func call, not func def
    pub fn func_args() -> Self {
        ExpectedToken::FuncArgs
    }

    pub fn doc_comment() -> Self {
        ExpectedToken::AnyDocComment
    }

    pub fn comma_or_gt() -> Self {
        ExpectedToken::Specific(vec![TokenKind::Punct(Punct::Comma), TokenKind::Punct(Punct::Gt)])
    }

    pub fn comma_or_colon() -> Self {
        ExpectedToken::Specific(vec![TokenKind::Punct(Punct::Comma), TokenKind::Punct(Punct::Colon)])
    }

    pub fn paren_brace_or_comma() -> Self {
        ExpectedToken::Specific(vec![
            TokenKind::Group {
                delim: Delim::Paren,
                tokens: vec![],
                prefix: b'\0',
            },
            TokenKind::Group {
                delim: Delim::Brace,
                tokens: vec![],
                prefix: b'\0',
            },
            TokenKind::Punct(Punct::Comma),
        ])
    }

    pub fn comma_or_paren() -> Self {
        ExpectedToken::Specific(vec![
            TokenKind::Punct(Punct::Comma),
            TokenKind::Group {
                delim: Delim::Paren,
                tokens: vec![],
                prefix: b'\0',
            },
        ])
    }

    pub fn guard_or_arrow() -> Self {
        ExpectedToken::Specific(vec![
            TokenKind::Punct(Punct::RArrow),
            TokenKind::Keyword(Keyword::If),
        ])
    }

    pub fn if_or_brace() -> Self {
        ExpectedToken::Specific(vec![
            TokenKind::Keyword(Keyword::If),
            TokenKind::Group {
                delim: Delim::Brace,
                tokens: vec![],
                prefix: b'\0',
            },
        ])
    }

    pub fn ident_or_brace() -> Self {
        ExpectedToken::IdentOrBrace
    }
}