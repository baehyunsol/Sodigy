use super::FuncDef;
use crate::ast::{ASTError, NameScope};
use crate::expr::Expr;
use crate::session::{InternedString, LocalParseSession};
use crate::span::Span;
use std::collections::{HashMap, HashSet};

pub struct Decorator {
    // a path consists of many names
    pub names: Vec<InternedString>,
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
    ) -> Result<(), ASTError> {

        match name_scope.search_name(self.names[0]) {
            Ok((Some(u), _)) => {
                if self.names.len() == 1 {
                    self.names = u.iter_path().map(|i| *i).collect();
                } else {
                    self.names = u.iter_path().chain(self.names[1..].iter()).map(|i| *i).collect();
                }
            },
            Ok((None, _)) => {},
            Err(_) => {
                return Err(ASTError::no_def(
                    self.names[0],
                    self.span,
                    name_scope.clone(),
                ));
            }
        }

        // we don't have to track names here
        let mut dummy = HashSet::new();

        for arg in self.args.iter_mut() {
            arg.resolve_names(name_scope, lambda_defs, session, &mut dummy)?;
        }

        Ok(())
    }

    pub fn dump(&self, session: &LocalParseSession) -> String {
        format!(
            "@{}{}",
            self.names.iter().map(
                |name| name.to_string(session)
            ).collect::<Vec<String>>().join("."),
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