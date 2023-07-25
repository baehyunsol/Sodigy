use crate::session::{InternedString, LocalParseSession};
use crate::span::Span;
use crate::stmt::{FuncDef, ModDef, Stmt, Use};
use std::collections::HashMap;

mod endec;
mod err;
mod name_resolve;
mod opt;

#[cfg(test)]
mod walker;

#[cfg(test)]
mod tests;

pub use err::ASTError;
pub use name_resolve::{NameOrigin, NameScope, NameScopeKind};

// It represents a single file.
// It doesn't have any data from other files, meaning that
// it's safe to reuse previously generated AST unless the file
// is modified.
pub struct AST {
    pub(crate) inner_modules: HashMap<InternedString, ModDef>,
    pub(crate) defs: HashMap<InternedString, FuncDef>,
    pub(crate) uses: HashMap<InternedString, Use>,
}

impl AST {

    // if it has an error, they're in `session`, but not returned
    pub(crate) fn from_stmts(stmts: Vec<Stmt>, session: &mut LocalParseSession) -> Result<Self, ()> {
        let mut curr_decos = vec![];
        let mut ast = AST {
            inner_modules: HashMap::new(),
            defs: HashMap::new(),
            uses: HashMap::new(),
        };

        let e = 'inner_error: {
            for stmt in stmts.into_iter() {

                match stmt {
                    Stmt::Decorator(d) => { curr_decos.push(d); }
                    Stmt::Def(mut f) => {
                        f.decorators = curr_decos;
                        curr_decos = vec![];

                        if let Some(first_def) = check_already_defined(&ast, &f.name) {
                            break 'inner_error Err(ASTError::multi_def(f.name, first_def, f.span));
                        }

                        else {
                            ast.defs.insert(f.name, f);
                        }

                    }
                    Stmt::Module(m) => {
                        if !curr_decos.is_empty() {
                            session.add_error(ASTError::deco_mod(m.span));
                            curr_decos = vec![];
                        }

                        if let Some(first_def) = check_already_defined(&ast, &m.name) {
                            break 'inner_error Err(ASTError::multi_def(m.name, first_def, m.span));
                        }

                        else {
                            ast.inner_modules.insert(m.name, m);
                        }
                    }
                    Stmt::Use(u) => {
                        if !curr_decos.is_empty() {
                            session.add_error(ASTError::deco_use(u.span));
                            curr_decos = vec![];
                        }

                        if let Some(first_def) = check_already_defined(&ast, &u.alias) {
                            break 'inner_error Err(ASTError::multi_def(u.alias, first_def, u.span));
                        }

                        else {
                            ast.uses.insert(u.alias, u);
                        }
                    }
                }

            }

            Ok(())
        };

        // Don't return here: we want to provide as many error messages as possible even though the compilation fails
        if let Err(e) = e {
            session.add_error(e);
        }

        ast.resolve_names(session)?;
        ast.resolve_recursive_funcs_in_block(session);
        ast.clean_up_blocks(session)?;
        ast.opt(session);

        Ok(ast)
    }

    pub fn dump(&self, session: &LocalParseSession) -> String {
        let mut result = Vec::with_capacity(
            self.defs.len() + self.uses.len() + self.inner_modules.len()
        );

        for module in self.inner_modules.values() {
            result.push(module.dump(session));
        }

        for use_ in self.uses.values() {
            result.push(use_.dump(session));
        }

        for def in self.defs.values() {
            result.push(def.dump(session));
        }

        result.join("\n\n")
    }

}

// if already defined, it returns the span of the previous definition
fn check_already_defined(ast: &AST, name: &InternedString) -> Option<Span> {
    if let Some(f) = ast.defs.get(name) {
        Some(f.span)
    }

    else if let Some(u) = ast.uses.get(name) {
        Some(u.span)
    }

    else if let Some(m) = ast.inner_modules.get(name) {
        Some(m.span)
    }

    else {
        None
    }
}
