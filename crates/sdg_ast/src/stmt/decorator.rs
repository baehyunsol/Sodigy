use super::FuncDef;
use crate::ast::NameScope;
use crate::expr::Expr;
use crate::path::Path;
use crate::session::{InternedString, LocalParseSession};
use crate::span::Span;
use std::collections::{HashMap, HashSet};

#[derive(Clone)]
pub struct Decorator {
    pub deco_name: Path,
    pub args: Vec<Expr>,

    // 0-args and no_args are different
    // `@deco` vs `@deco()`
    pub no_args: bool,

    // of `@`
    pub span: Span,
}

impl Decorator {
    pub fn resolve_names(
        &mut self,
        name_scope: &mut NameScope,
        lambda_defs: &mut HashMap<InternedString, FuncDef>,
        session: &mut LocalParseSession,
    ) {
        self.deco_name.resolve_names(name_scope, session);

        // we don't have to track names here
        let mut dummy = HashSet::new();

        for arg in self.args.iter_mut() {
            arg.resolve_names(name_scope, lambda_defs, session, &mut dummy);
        }
    }

    pub fn dump(&self, session: &LocalParseSession) -> String {
        #[cfg(test)]
        assert_eq!(self.span.dump(session), "@");

        format!(
            "@{}{}",
            self.deco_name.dump(session),
            if self.no_args {
                String::new()
            } else {
                format!(
                    "({})",
                    self.args.iter().map(
                        |arg| arg.dump(session)
                    ).collect::<Vec<String>>().join(", ")
                )
            }
        )
    }
}