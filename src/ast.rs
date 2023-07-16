use crate::session::InternedString;
use crate::stmt::{FuncDef, Stmt, StmtKind, Use};
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

    pub(crate) fn from_stmts(stmts: Vec<Stmt>) -> Result<Self, ASTError> {
        let mut curr_decos = vec![];
        let mut ast = AST {
            defs: HashMap::new(),
            uses: HashMap::new(),
        };

        for stmt in stmts.into_iter() {

            match stmt.kind {
                StmtKind::Decorator(d) => { curr_decos.push(d); }
                StmtKind::Def(mut f) => {
                    f.decorators = curr_decos;
                    curr_decos = vec![];

                    if ast.defs.contains_key(&f.name) {
                        let first_def = ast.defs.get(&f.name).expect(
                            "Internal Compiler Error 3E1BDDB"
                        ).span;
                        return Err(ASTError::multi_def(f.name, first_def, f.span));
                    }

                    else if ast.uses.contains_key(&f.name) {
                        let first_def = ast.uses.get(&f.name).expect(
                            "Internal Compiler Error 0A7DF53"
                        ).span;
                        return Err(ASTError::multi_def(f.name, first_def, f.span));
                    }

                    else {
                        ast.defs.insert(f.name, f);
                    }

                }
                StmtKind::Use(u) => {

                    if !curr_decos.is_empty() {
                        return Err(ASTError::deco(u.span));
                    }

                    if ast.defs.contains_key(&u.alias) {
                        let first_def = ast.defs.get(&u.alias).expect(
                            "Internal Compiler Error 12D24D5"
                        ).span;
                        return Err(ASTError::multi_def(u.alias, first_def, u.span));
                    }

                    else if ast.uses.contains_key(&u.alias) {
                        let first_def = ast.uses.get(&u.alias).expect(
                            "Internal Compiler Error 035B6A1"
                        ).span;
                        return Err(ASTError::multi_def(u.alias, first_def, u.span));
                    }

                    else {
                        ast.uses.insert(u.alias, u);
                    }

                }
            }

        }

        ast.resolve_names()?;
        ast.opt()?;

        Ok(ast)
    }

}