use crate::ast::NameOrigin;
use crate::session::{InternedString, LocalParseSession};
use crate::span::Span;
use crate::value::ValueKind;
use sdg_uid::UID;

mod kind;
mod ops;
mod parse;
mod name_resolve;

#[cfg(test)]
mod tests;

pub use kind::{ExprKind, MatchBranch};
pub use ops::{InfixOp, PostfixOp, PrefixOp};
pub use parse::{parse_match_body, parse_expr};

#[cfg(test)]
pub use tests::dump_ast_of_expr;

// `span` points to the first character of the operator
#[derive(Clone)]
pub struct Expr {
    pub span: Span,
    pub kind: ExprKind,
}

impl Expr {
    pub fn is_block_with_0_defs(&self) -> bool {
        match &self.kind {
            ExprKind::Value(ValueKind::Block { defs, .. }) if defs.is_empty() => true,
            _ => false,
        }
    }

    pub fn unwrap_block_value(&self) -> Expr {
        match &self.kind {
            ExprKind::Value(ValueKind::Block { value, .. }) => *value.clone(),
            _ => panic!("Internal Compiler Error 0687238F1E8"),
        }
    }

    pub fn is_closure(&self) -> bool {
        self.kind.is_closure()
    }

    pub fn unwrap_lambda_name(&self) -> InternedString {
        self.kind.unwrap_lambda_name()
    }

    pub fn is_lambda(&self) -> bool {
        self.kind.is_lambda()
    }

    pub fn unwrap_closure_name(&self) -> InternedString {
        self.kind.unwrap_closure_name()
    }

    pub fn new_identifier(name: InternedString, origin: NameOrigin, span: Span) -> Self {
        Expr {
            kind: ExprKind::Value(ValueKind::Identifier(name, origin)),
            span,
        }
    }

    pub fn new_object(id: UID, span: Span) -> Self {
        Expr {
            kind: ExprKind::Value(ValueKind::Object(id)),
            span,
        }
    }

    pub fn new_call(f: Expr, args: Vec<Expr>, span: Span) -> Self {
        Expr {
            kind: ExprKind::Call(Box::new(f), args),
            span,
        }
    }

    // TODO: it should belong to another file
    pub fn new_type_instance(type_id: UID) -> Self {
        Expr {
            // TODO: how should I impl it?
            kind: ExprKind::Value(ValueKind::Integer(type_id.to_u128().into())),
            span: Span::dummy(),
        }
    }

    pub fn dump(&self, session: &LocalParseSession) -> String {
        self.kind.dump(session, self.span)
    }
}
