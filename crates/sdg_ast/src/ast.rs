use crate::session::{InternedString, LocalParseSession};
use crate::stmt::{FuncDef, Stmt, Use};
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
    pub(crate) defs: HashMap<InternedString, FuncDef>,
    pub(crate) uses: HashMap<InternedString, Use>,
}

impl AST {

    // if it has an error, they're in `session`, but not returned
    pub(crate) fn from_stmts(stmts: Vec<Stmt>, session: &mut LocalParseSession) -> Result<Self, ()> {
        let mut curr_decos = vec![];
        let mut ast = AST {
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

                        if ast.defs.contains_key(&f.name) {
                            let first_def = ast.defs.get(&f.name).expect(
                                "Internal Compiler Error 32C4175D312"
                            ).span;
                            break 'inner_error Err(ASTError::multi_def(f.name, first_def, f.span));
                        }

                        else if ast.uses.contains_key(&f.name) {
                            let first_def = ast.uses.get(&f.name).expect(
                                "Internal Compiler Error 141662FE076"
                            ).span;
                            break 'inner_error Err(ASTError::multi_def(f.name, first_def, f.span));
                        }

                        else {
                            ast.defs.insert(f.name, f);
                        }

                    }
                    Stmt::Use(u) => {

                        if !curr_decos.is_empty() {
                            break 'inner_error Err(ASTError::deco(u.span));
                        }

                        if ast.defs.contains_key(&u.alias) {
                            let first_def = ast.defs.get(&u.alias).expect(
                                "Internal Compiler Error DD2D5DD058A"
                            ).span;
                            break 'inner_error Err(ASTError::multi_def(u.alias, first_def, u.span));
                        }

                        else if ast.uses.contains_key(&u.alias) {
                            let first_def = ast.uses.get(&u.alias).expect(
                                "Internal Compiler Error 56D7C654ADC"
                            ).span;
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

        if let Err(e) = e {
            session.add_error(e);
        }

        ast.resolve_names(session)?;
        ast.clean_up_blocks(session)?;
        ast.opt(session);

        Ok(ast)
    }

}