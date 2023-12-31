#![deny(unused_imports)]

use sodigy_intern::InternedString;
use sodigy_span::SpanRange;
use sodigy_uid::Uid;

mod endec;
mod error;
mod expr;
mod fmt;
mod let_;
mod ops;
mod parse;
mod pattern;
mod session;
mod stmt;
mod tokens;
mod utils;
mod value;
mod warn;

#[cfg(test)]
mod tests;

pub use expr::{Expr, ExprKind};
pub use let_::{Let, LetKind};
pub use ops::{InfixOp, PostfixOp, PrefixOp};
pub use parse::{parse_expr, parse_stmts};
pub use pattern::{PatField, Pattern, PatternKind};
pub use session::AstSession;
pub use stmt::*;
pub use tokens::Tokens;
pub use value::ValueKind;

pub use sodigy_parse::{TokenTree as Token, TokenTreeKind as TokenKind};

#[derive(Clone, Copy, Debug, Eq, Hash, PartialEq)]
pub struct IdentWithSpan(InternedString, SpanRange);

impl IdentWithSpan {
    pub fn new(id: InternedString, span: SpanRange) -> Self {
        IdentWithSpan(id, span)
    }

    pub fn id(&self) -> InternedString {
        self.0
    }

    pub fn span(&self) -> &SpanRange {
        &self.1
    }
}

pub type DottedNames = Vec<IdentWithSpan>;

#[derive(Clone, Debug)]
pub struct ArgDef {
    pub name: IdentWithSpan,
    pub ty: Option<TypeDef>,
    pub has_question_mark: bool,
    pub attributes: Vec<Attribute>,
}

impl ArgDef {
    pub fn has_type(&self) -> bool {
        self.ty.is_some()
    }
}

#[derive(Clone, Debug)]
pub struct ScopeBlock {
    pub lets: Vec<Let>,
    pub value: Box<Expr>,
}

impl ScopeBlock {
    pub fn has_no_lets(&self) -> bool {
        self.lets.is_empty()
    }
}

// for now, a type is a comp-time evaluable expression, whose type is `Type`.
#[derive(Clone, Debug)]
pub struct TypeDef(pub Expr);

impl TypeDef {
    pub fn from_expr(e: Expr) -> Self {
        TypeDef(e)
    }
}

pub type GenericDef = IdentWithSpan;

#[derive(Clone, Debug)]
pub struct MatchArm {
    pub pattern: Pattern,
    pub guard: Option<Expr>,
    pub value: Expr,
    pub uid: Uid,
}

#[derive(Clone, Debug)]
pub struct BranchArm {
    pub span: SpanRange,  // merged span of `if`, `else` and `pattern` keywords
    pub cond: Option<Expr>,
    pub pattern_bind: Option<Pattern>,  // `if pattern` pattern_bind = cond { value }
    pub value: Expr,
}

#[derive(Clone, Debug)]
pub struct StructInitDef {
    pub field: IdentWithSpan,
    pub value: Expr,
}
