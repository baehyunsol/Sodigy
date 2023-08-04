use super::ArgDef;
use crate::ast::NameOrigin;
use crate::expr::Expr;
use crate::session::{InternedString, LocalParseSession};
use crate::span::Span;
use sdg_uid::UID;

#[cfg(test)]
use crate::utils::assert_identifier;

#[derive(Clone)]
pub struct GenericDef {
    pub(crate) name: InternedString,
    pub(crate) span: Span,
}

impl GenericDef {
    pub fn new(name: InternedString, span: Span) -> Self {
        GenericDef { name, span }
    }

    pub fn to_arg_def(&self) -> ArgDef {
        ArgDef {
            name: self.name,
            span: self.span,
            ty: Some(Expr::new_object(sdg_uid::prelude::type_(), Span::dummy())),
        }
    }

    pub fn to_expr(&self, parent_id: UID) -> Expr {
        Expr::new_identifier(self.name, NameOrigin::GenericArg(parent_id), self.span)
    }

    pub fn dump(&self, session: &LocalParseSession) -> String {
        #[cfg(test)]
        assert_identifier(self.span.dump(session));

        self.name.to_string(session)
    }
}
