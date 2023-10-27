use sodigy_err::ErrorContext;
use sodigy_intern::InternedString;
use sodigy_parse::FormattedStringElement;
use sodigy_span::SpanRange;

mod err;
mod expr;
mod ops;
mod parse;
mod session;
mod stmt;
mod tokens;
mod value;

#[cfg(test)]
mod tests;

pub use expr::{Expr, ExprKind};
pub use parse::{parse_expr, parse_stmts};
pub use session::AstSession;
pub use stmt::*;
pub use tokens::Tokens;
use value::ValueKind;

pub use sodigy_parse::{TokenTree as Token, TokenTreeKind as TokenKind};

#[derive(Clone, Copy, Debug)]
pub struct IdentWithSpan(InternedString, SpanRange);

impl IdentWithSpan {
    pub fn new(id: InternedString, span: SpanRange) -> Self {
        IdentWithSpan(id, span)
    }

    pub fn id(&self) -> &InternedString {
        &self.0
    }

    pub fn span(&self) -> &SpanRange {
        &self.1
    }
}

#[derive(Clone)]
pub struct ArgDef {
    pub name: IdentWithSpan,
    pub ty: Option<TypeDef>,
    pub has_question_mark: bool,
}

impl ArgDef {
    pub fn has_type(&self) -> bool {
        self.ty.is_some()
    }
}

#[derive(Clone)]
pub struct ScopeDef {
    pub defs: Vec<LocalDef>,
    pub value: Box<Expr>,
}

#[derive(Clone)]
pub struct LocalDef {
    pub let_span: SpanRange,
    pub pattern: Pattern,
    pub value: Expr,
}

// for now, a type is a comp-time evaluable expression, whose type is `Type`.
#[derive(Clone)]
pub struct TypeDef(Expr);

impl TypeDef {
    pub fn from_expr(e: Expr) -> Self {
        TypeDef(e)
    }
}

pub type GenericDef = IdentWithSpan;

#[derive(Clone)]
pub struct MatchArm {
    pub pattern: Pattern,
    pub guard: Option<Expr>,
    pub value: Expr,
}

#[derive(Clone)]
pub struct BranchArm {
    pub cond: Option<Expr>,
    pub value: Expr,
}

#[derive(Clone)]
pub struct Pattern {}

#[derive(Clone)]
pub struct StructInitDef {
    pub field: IdentWithSpan,
    pub value: Expr,
}

// TODO: where should this function belong?
fn format_string_into_expr(
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
