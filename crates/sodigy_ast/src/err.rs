use crate::{IdentWithSpan, Token, TokenKind};
use crate::pattern::{Pattern, PatternKind};
use smallvec::{smallvec, SmallVec};
use sodigy_err::{substr_edit_distance, ErrorContext, ExtraErrInfo, SodigyError, SodigyErrorKind};
use sodigy_intern::{InternedString, InternSession};
use sodigy_keyword::Keyword;
use sodigy_parse::Delim;
use sodigy_span::SpanRange;

mod fmt;

const STMT_START_KEYWORDS: [&'static str; 5] = [
    "def", "enum", "struct", "module", "import",
];

#[derive(Clone)]
pub struct AstError {
    pub(crate) kind: AstErrorKind,
    spans: SmallVec<[SpanRange; 1]>,
    extra: ExtraErrInfo,
}

impl AstError {
    pub fn unexpected_token(token: Token, expected_token: ExpectedToken) -> Self {
        let mut extra = ExtraErrInfo::none();

        match &token.kind {
            TokenKind::Keyword(k) => {
                // expected ident, but got keyword K
                // tell them that K is not an identifier
                match expected_token {
                    ExpectedToken::AnyIdentifier
                    | ExpectedToken::AnyExpression
                    | ExpectedToken::IdentOrBrace => {
                        extra.set_message(format!(
                            "`{k}` is a keyword, not an identifier. If you want to use `{k}` as an identifier, try `{k}_`."
                        ));
                    },
                    _ => {},
                }
            },
            TokenKind::Identifier(id) => {
                match expected_token {
                    // This is very expensive. Make sure that compilation has already failed before this branch is reached.
                    ExpectedToken::AnyStatement => {
                        let mut sess = InternSession::new();
                        let id = match sess.unintern_string(*id) {
                            Some(s) => s.to_vec(),
                            _ => b"Unexpected error, but I don't want it to mess up any other stuff.".to_vec(),
                        };

                        for stmt_start in STMT_START_KEYWORDS.iter() {
                            if substr_edit_distance(&id, stmt_start.as_bytes()) < 2 {
                                extra.set_message(format!("Did you mean `{stmt_start}`?"));
                            }
                        }
                    },
                    _ => {},
                }
            },
            TokenKind::Group {
                delim: Delim::Brace,
                prefix: b'\0',
                tokens,
            } if tokens.is_empty() => {
                extra.set_message(String::from("It's obvious that it's an error, but it's hard to know what you've intended. If you're to initialize a struct, please provide fields. A struct in Sodigy must have at least one field. If it's just an expression, please provide a value."));
            },
            _ => {},
        }

        if !extra.has_message() {
            match expected_token {
                ExpectedToken::AnyStatement => {
                    extra.set_message(String::from("Sodigy is not a script language. If you want to execute something, please use `main`."));
                },
                _ => {},
            }
        }

        AstError {
            kind: AstErrorKind::UnexpectedToken(token.kind, expected_token),
            spans: smallvec![token.span],
            extra,
        }
    }

    pub fn unexpected_end(span: SpanRange, expected_token: ExpectedToken) -> Self {
        AstError {
            kind: AstErrorKind::UnexpectedEnd(expected_token),
            spans: smallvec![span],
            extra: ExtraErrInfo::none(),
        }
    }

    pub fn empty_generic_list(span: SpanRange) -> Self {
        AstError {
            kind: AstErrorKind::EmptyGenericList,
            spans: smallvec![span],
            extra: ExtraErrInfo::none(),
        }
    }

    pub fn binary_char(span: SpanRange) -> Self {
        AstError {
            kind: AstErrorKind::BinaryChar,
            spans: smallvec![span],
            extra: ExtraErrInfo::none(),
        }
    }

    pub fn empty_char_literal(span: SpanRange) -> Self {
        AstError {
            kind: AstErrorKind::EmptyCharLiteral,
            spans: smallvec![span],
            extra: ExtraErrInfo::none(),
        }
    }

    pub fn too_long_char_literal(span: SpanRange) -> Self {
        AstError {
            kind: AstErrorKind::TooLongCharLiteral,
            spans: smallvec![span],
            extra: ExtraErrInfo::none(),
        }
    }

    pub fn empty_scope_block(span: SpanRange) -> Self {
        AstError {
            kind: AstErrorKind::EmptyScopeBlock,
            spans: smallvec![span],
            extra: ExtraErrInfo::none(),
        }
    }

    pub fn empty_match_body(span: SpanRange) -> Self {
        AstError {
            kind: AstErrorKind::EmptyMatchBody,
            spans: smallvec![span],
            extra: ExtraErrInfo::at_context(ErrorContext::ParsingMatchBody),
        }
    }

    pub fn empty_struct_body(span: SpanRange) -> Self {
        AstError {
            kind: AstErrorKind::EmptyStructBody,
            spans: smallvec![span],
            extra: ExtraErrInfo::at_context(ErrorContext::ParsingStructBody),
        }
    }

    pub fn func_arg_without_type(func_name: InternedString, arg: IdentWithSpan) -> Self {
        AstError {
            kind: AstErrorKind::FuncArgWithoutType { arg_name: *arg.id(), func_name },
            spans: smallvec![*arg.span()],
            extra: ExtraErrInfo::at_context(ErrorContext::ParsingFuncArgs),
        }
    }

    pub fn expected_binding_got_pattern(pat: Pattern) -> Self {
        AstError {
            kind: AstErrorKind::ExpectedBindingGotPattern(pat.kind),
            spans: smallvec![pat.span],
            extra: ExtraErrInfo::at_context(ErrorContext::ParsingPattern),
        }
    }

    pub fn multiple_shorthands_in_one_pattern(spans: SmallVec<[SpanRange; 1]>) -> Self {
        AstError {
            kind: AstErrorKind::MultipleShorthandsInOnePattern,
            spans,
            extra: ExtraErrInfo::at_context(ErrorContext::ParsingPattern),
        }
    }

    pub fn invalid_utf8(span: SpanRange) -> Self {
        AstError {
            kind: AstErrorKind::InvalidUtf8,
            spans: smallvec![span],
            extra: ExtraErrInfo::none(),
        }
    }

    pub fn todo(msg: &str, span: SpanRange) -> Self {
        AstError {
            kind: AstErrorKind::TODO(msg.to_string()),
            spans: smallvec![span],
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

    fn index(&self) -> u32 {
        3
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
    EmptyStructBody,
    FuncArgWithoutType { arg_name: InternedString, func_name: InternedString },
    ExpectedBindingGotPattern(PatternKind),
    MultipleShorthandsInOnePattern,
    InvalidUtf8,
    TODO(String),
}

impl SodigyErrorKind for AstErrorKind {
    fn msg(&self, _: &mut InternSession) -> String {
        match self {
            AstErrorKind::UnexpectedToken(token, expected) => format!("expected {expected}, got `{}`", token.render_error()),
            AstErrorKind::UnexpectedEnd(expected) => format!("expected {expected}, got nothing"),
            AstErrorKind::EmptyGenericList => String::from("empty generic parameter list"),
            AstErrorKind::BinaryChar => String::from("binary character literal"),
            AstErrorKind::EmptyCharLiteral => String::from("empty character literal"),
            AstErrorKind::TooLongCharLiteral => String::from("too long character literal"),
            AstErrorKind::EmptyScopeBlock => String::from("expected expressions or local values, got nothing"),
            AstErrorKind::EmptyMatchBody => String::from("expected patterns, got nothing"),
            AstErrorKind::EmptyStructBody => String::from("expected fields, got nothing"),
            AstErrorKind::FuncArgWithoutType { .. } => String::from("a function argument without a type annotation"),
            AstErrorKind::ExpectedBindingGotPattern(p) => format!("expected a name binding, get pattern `{}`", p.render_error()),
            AstErrorKind::MultipleShorthandsInOnePattern => String::from("multiple shorthands in one pattern"),
            AstErrorKind::InvalidUtf8 => String::from("invalid utf-8"),
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
            AstErrorKind::EmptyStructBody => String::from("A struct must have at least one field."),
            AstErrorKind::EmptyGenericList => String::from("Try remove angle brackets."),
            AstErrorKind::BinaryChar => String::from("Try remove prefix `b`."),
            AstErrorKind::FuncArgWithoutType { arg_name, func_name } => format!(
                "Argument `{arg_name}` of `{func_name}` needs a type annotation."
            ),
            AstErrorKind::EmptyCharLiteral
            | AstErrorKind::TooLongCharLiteral => String::from("If you meant a string literal, use double quotes."),
            AstErrorKind::ExpectedBindingGotPattern(p) => match p {
                PatternKind::Identifier(id) => format!(
                    "`{id}` is a name, not a name binding. Try `${id}` to bind a name.",
                ),
                _ => String::new(),
            },
            AstErrorKind::MultipleShorthandsInOnePattern
            | AstErrorKind::InvalidUtf8 => String::new(),
        }
    }

    fn index(&self) -> u32 {
        match self {
            AstErrorKind::UnexpectedToken(..) => 0,
            AstErrorKind::UnexpectedEnd(..) => 1,
            AstErrorKind::EmptyGenericList => 2,
            AstErrorKind::BinaryChar => 3,
            AstErrorKind::EmptyCharLiteral => 4,
            AstErrorKind::TooLongCharLiteral => 5,
            AstErrorKind::EmptyScopeBlock => 6,
            AstErrorKind::EmptyMatchBody => 7,
            AstErrorKind::EmptyStructBody => 8,
            AstErrorKind::FuncArgWithoutType { .. } => 9,
            AstErrorKind::ExpectedBindingGotPattern(..) => 10,
            AstErrorKind::MultipleShorthandsInOnePattern => 11,
            AstErrorKind::InvalidUtf8 => 12,
            AstErrorKind::TODO(..) => 63,
        }
    }
}

#[derive(Clone)]
pub enum ExpectedToken {
    AnyIdentifier,
    AnyExpression,
    AnyStatement,
    AnyDocComment,
    AnyPattern,
    AnyType,
    AnyNumber,
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

    pub fn pattern() -> Self {
        ExpectedToken::AnyPattern
    }

    pub fn ty() -> Self {
        ExpectedToken::AnyType
    }

    pub fn number() -> Self {
        ExpectedToken::AnyNumber
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
        ExpectedToken::Specific(vec![TokenKind::comma(), TokenKind::gt()])
    }

    pub fn comma_or_colon() -> Self {
        ExpectedToken::Specific(vec![TokenKind::comma(), TokenKind::colon()])
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
            TokenKind::comma(),
        ])
    }

    pub fn comma_or_paren() -> Self {
        ExpectedToken::Specific(vec![
            TokenKind::comma(),
            TokenKind::Group {
                delim: Delim::Paren,
                tokens: vec![],
                prefix: b'\0',
            },
        ])
    }

    pub fn guard_or_arrow() -> Self {
        ExpectedToken::Specific(vec![
            TokenKind::r_arrow(),
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

    pub fn comma_semicolon_dot_or_from() -> Self {
        ExpectedToken::Specific(vec![
            TokenKind::comma(),
            TokenKind::semi_colon(),
            TokenKind::dot(),
            TokenKind::Keyword(Keyword::From),
        ])
    }
}
