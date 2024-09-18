use crate::{Expr, IdentWithSpan, Token, TokenKind};
use crate::pattern::{Pattern, PatternKind};
use smallvec::{SmallVec, smallvec};
use sodigy_attribute::Attribute;
use sodigy_error::{
    ErrorContext,
    ExpectedToken,
    ExtraErrInfo,
    SodigyError,
    SodigyErrorKind,
    Stage,
    substr_edit_distance,
};
use sodigy_error::RenderError;
use sodigy_intern::{InternedString, InternSession};
use sodigy_keyword::Keyword;
use sodigy_parse::Delim;
use sodigy_span::SpanRange;

const STMT_START_KEYWORDS: [&'static str; 3] = [
    "let", "module", "import",
];

#[derive(Clone, Debug)]
pub struct AstError {
    pub(crate) kind: AstErrorKind,
    spans: SmallVec<[SpanRange; 1]>,
    extra: ExtraErrInfo,
}

impl AstError {
    pub fn unexpected_token(token: Token, expected_token: ExpectedToken<TokenKind>) -> Self {
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
                        let id = sess.unintern_string(*id).to_vec();

                        if id == b"fn" || id == b"def" {
                            extra.set_message(format!("Do you mean `let`?"));
                        }

                        else {
                            for stmt_start in STMT_START_KEYWORDS.iter() {
                                if substr_edit_distance(&id, stmt_start.as_bytes()) < 2 {
                                    extra.set_message(format!("Do you mean `{stmt_start}`?"));
                                }
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
                    extra.set_message(String::from("Sodigy is not a script language. If you want to execute something, try `let main = ...;`."));
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

    pub fn unexpected_end(span: SpanRange, expected_token: ExpectedToken<TokenKind>) -> Self {
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
            kind: AstErrorKind::FuncArgWithoutType { arg_name: arg.id(), func_name },
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

    pub fn no_generics_allowed(span: SpanRange) -> Self {
        AstError {
            kind: AstErrorKind::NoGenericsAllowed,
            spans: smallvec![span],
            extra: ExtraErrInfo::none(),
        }
    }

    pub fn stranded_attribute(attributes: Vec<Attribute<Expr>>, ctxt: AttributeIn) -> Self {
        AstError {
            kind: AstErrorKind::StrandedAttribute { ctxt, multiple_attributes: attributes.len() > 1 },
            spans: attributes.iter().map(|attr| attr.span()).collect(),
            extra: ExtraErrInfo::at_context(ErrorContext::ParsingFuncArgs),
        }
    }

    pub fn name_binding_not_allowed(binding_span: SpanRange) -> Self {
        AstError {
            kind: AstErrorKind::NameBindingNotAllowed,
            spans: smallvec![binding_span],
            extra: ExtraErrInfo::none(),
        }
    }

    pub fn type_anno_not_allowed(ty_span: SpanRange) -> Self {
        AstError {
            kind: AstErrorKind::TypeAnnoNotAllowed,
            spans: smallvec![ty_span],
            extra: ExtraErrInfo::none(),
        }
    }

    pub fn excessive_or_pattern(pattern_span: SpanRange, limit: usize) -> Self {
        AstError {
            kind: AstErrorKind::ExcessiveOrPattern { limit },
            spans: smallvec![pattern_span],
            extra: ExtraErrInfo::none(),
        }
    }

    pub fn invalid_utf8(span: SpanRange) -> Self {
        AstError {
            kind: AstErrorKind::InvalidUtf8,
            spans: smallvec![span],
            extra: ExtraErrInfo::none(),
        }
    }

    pub fn add_expected_token(&mut self, token: TokenKind) -> Result<(), ()> {
        match &mut self.kind {
            AstErrorKind::UnexpectedToken(_, tokens)
            | AstErrorKind::UnexpectedEnd(tokens) => tokens.add_specific_token(token),
            _ => Err(()),
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

    fn get_first_span(&self) -> Option<SpanRange> {
        self.spans.get(0).copied()
    }

    fn get_spans(&self) -> &[SpanRange] {
        &self.spans
    }

    fn error_kind(&self) -> &AstErrorKind {
        &self.kind
    }

    fn index(&self) -> u32 {
        0
    }

    fn get_stage(&self) -> Stage {
        Stage::Ast
    }
}

#[derive(Clone, Debug)]
pub enum AstErrorKind {
    UnexpectedToken(TokenKind, ExpectedToken<TokenKind>),
    UnexpectedEnd(ExpectedToken<TokenKind>),
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
    NoGenericsAllowed,
    StrandedAttribute {
        // these fields help making nicer error messages
        ctxt: AttributeIn,
        multiple_attributes: bool,
    },
    NameBindingNotAllowed,
    TypeAnnoNotAllowed,
    ExcessiveOrPattern { limit: usize },
    InvalidUtf8,
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
            AstErrorKind::NoGenericsAllowed => String::from("generic parameter not allowed here"),
            AstErrorKind::StrandedAttribute { ctxt, multiple_attributes } => {
                let (a, s) = if *multiple_attributes {
                    ("", "s")
                } else {
                    ("a ", "")
                };

                let ctxt = match ctxt {
                    AttributeIn::TopLevel => " source code",
                    AttributeIn::FuncArg => " function argument list",
                    AttributeIn::ScopedLet => " scoped block",
                    AttributeIn::Enum => "n enum body",
                    AttributeIn::Struct => " struct body",
                };

                format!("{a}stranded attribute{s} in a{ctxt}")
            },
            AstErrorKind::NameBindingNotAllowed => String::from("name binding not allowed in this place"),
            AstErrorKind::TypeAnnoNotAllowed => String::from("type annotation not allowed in this place"),
            AstErrorKind::ExcessiveOrPattern { .. } => String::from("excessive use of `|` in a pattern"),
            AstErrorKind::InvalidUtf8 => String::from("invalid utf-8"),
        }
    }

    fn help(&self, _: &mut InternSession) -> String {
        match self {
            AstErrorKind::UnexpectedToken(_, _)
            | AstErrorKind::UnexpectedEnd(_)
            | AstErrorKind::EmptyScopeBlock
            | AstErrorKind::EmptyMatchBody => String::new(),
            AstErrorKind::EmptyStructBody => String::from("A struct must have at least one field."),
            AstErrorKind::EmptyGenericList => String::from("Try remove angle brackets."),
            AstErrorKind::BinaryChar => String::from("Try remove prefix `b`."),
            AstErrorKind::FuncArgWithoutType { arg_name, func_name } => format!(
                "Argument `{}` of `{}` needs a type annotation.",
                arg_name.render_error(),
                func_name.render_error(),
            ),
            AstErrorKind::EmptyCharLiteral
            | AstErrorKind::TooLongCharLiteral => String::from("If you meant a string literal, use double quotes."),
            AstErrorKind::ExpectedBindingGotPattern(p) => match p {
                PatternKind::Identifier(id) => format!(
                    "`{}` is a name, not a name binding. Try `${}` to bind a name.",
                    id.render_error(),
                    id.render_error(),
                ),
                _ => String::new(),
            },
            AstErrorKind::StrandedAttribute { multiple_attributes, .. } => {
                let (this, s, does) = if *multiple_attributes {
                    ("These", "s", "do")
                } else {
                    ("This", "", "does")
                };

                format!("{this} attribute{s} {does}n't do anything.")
            },
            AstErrorKind::NoGenericsAllowed => String::from("Generic parameters are only allowed in top-level statements."),
            AstErrorKind::ExcessiveOrPattern { limit } => format!("The compiler naively expands `|` operators in patterns, and it might lead to an exponential blow-up. The current limit is {limit} and you can adjust it with `--or-pattern-limit` command-line option."),
            AstErrorKind::MultipleShorthandsInOnePattern
            | AstErrorKind::InvalidUtf8
            | AstErrorKind::NameBindingNotAllowed
            | AstErrorKind::TypeAnnoNotAllowed => String::new(),
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
            AstErrorKind::NoGenericsAllowed => 12,
            AstErrorKind::StrandedAttribute { .. } => 13,
            AstErrorKind::NameBindingNotAllowed => 14,
            AstErrorKind::TypeAnnoNotAllowed => 15,
            AstErrorKind::ExcessiveOrPattern { .. } => 16,
            AstErrorKind::InvalidUtf8 => 17,
        }
    }
}

#[derive(Clone, Debug)]
pub enum AttributeIn {
    TopLevel,
    ScopedLet,
    FuncArg,

    // Those 2 are not instantiated:
    // enum and struct bodies are parsed in a different way
    Enum,
    Struct,
}

// walk-around: the rust compiler doesn't allow me to define
// `ExpectedToken`'s methods otherwise
pub trait NewExpectedTokens {
    fn comma_or_gt() -> Self;
    fn comma_or_colon() -> Self;
    fn paren_brace_or_comma() -> Self;
    fn comma_or_paren() -> Self;
    fn guard_or_arrow() -> Self;
    fn if_or_brace() -> Self;
    fn comma_semicolon_dot_or_from() -> Self;
}

impl NewExpectedTokens for ExpectedToken<TokenKind> {
    fn comma_or_gt() -> Self {
        ExpectedToken::Specific(vec![TokenKind::comma(), TokenKind::gt()])
    }

    fn comma_or_colon() -> Self {
        ExpectedToken::Specific(vec![TokenKind::comma(), TokenKind::colon()])
    }

    fn paren_brace_or_comma() -> Self {
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

    fn comma_or_paren() -> Self {
        ExpectedToken::Specific(vec![
            TokenKind::comma(),
            TokenKind::Group {
                delim: Delim::Paren,
                tokens: vec![],
                prefix: b'\0',
            },
        ])
    }

    fn guard_or_arrow() -> Self {
        ExpectedToken::Specific(vec![
            TokenKind::r_arrow(),
            TokenKind::Keyword(Keyword::If),
        ])
    }

    fn if_or_brace() -> Self {
        ExpectedToken::Specific(vec![
            TokenKind::Keyword(Keyword::If),
            TokenKind::Group {
                delim: Delim::Brace,
                tokens: vec![],
                prefix: b'\0',
            },
        ])
    }

    fn comma_semicolon_dot_or_from() -> Self {
        ExpectedToken::Specific(vec![
            TokenKind::comma(),
            TokenKind::semi_colon(),
            TokenKind::dot(),
            TokenKind::Keyword(Keyword::From),
        ])
    }
}
