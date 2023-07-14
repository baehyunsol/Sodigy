use crate::ast::{ASTError, NameScope};
use crate::expr::Expr;
use crate::session::InternedString;
use crate::span::Span;

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

    pub fn resolve_names(&mut self, name_scope: &mut NameScope) -> Result<(), ASTError> {

        match name_scope.search_name(self.names[0]) {
            Ok(Some(u)) => {
                if self.names.len() == 1 {
                    self.names = u.iter_path().map(|i| *i).collect();
                } else {
                    self.names = u.iter_path().chain(self.names[1..].iter()).map(|i| *i).collect();
                }
            },
            Ok(None) => {},
            Err(_) => {
                return Err(ASTError::no_def(
                    self.names[0],
                    self.span,
                    name_scope.clone(),
                ));
            }
        }

        for arg in self.args.iter_mut() {
            arg.resolve_names(name_scope)?;
        }

        Ok(())
    }

}