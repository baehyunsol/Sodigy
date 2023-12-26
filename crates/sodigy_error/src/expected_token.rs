mod endec;

#[derive(Clone, Debug)]
pub enum ExpectedToken<T> {
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
    Specific(Vec<T>),
    LetStatement,
}

impl<T> ExpectedToken<T> {
    pub fn specific(t: T) -> Self {
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

    pub fn ident_or_brace() -> Self {
        ExpectedToken::IdentOrBrace
    }

    pub fn let_statement() -> Self {
        ExpectedToken::LetStatement
    }
}
