use crate::session::{InternedString, LocalParseSession};
use crate::span::Span;
use crate::stmt::{FuncDef, ModDef, Stmt, Use};
use std::collections::HashMap;

mod endec;
mod err;
mod name_resolve;
mod opt;
mod walker;

use opt::ClosureCollector;

#[cfg(test)]
mod tests;

pub use err::ASTError;
pub use name_resolve::{NameOrigin, NameScope, NameScopeKind};
pub use opt::{LocalUIDs, Opt};

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
        let curr_location = session.curr_name_path().clone();

        for stmt in stmts.into_iter() {

            match stmt {
                Stmt::Decorator(d) => { curr_decos.push(d); }
                Stmt::Def(mut f) => {
                    f.decorators = curr_decos;
                    f.set_location(&curr_location);
                    curr_decos = vec![];

                    if let Some(first_def) = check_already_defined(&ast, &f.name) {
                        session.add_error(ASTError::multi_def(f.name, first_def, f.name_span));
                    }

                    else {
                        ast.defs.insert(f.name, f);
                    }

                }
                Stmt::Module(m) => {
                    if !curr_decos.is_empty() {
                        session.add_error(ASTError::deco_mod(m.def_span));
                        curr_decos = vec![];
                    }

                    if let Some(first_def) = check_already_defined(&ast, &m.name) {
                        session.add_error(ASTError::multi_def(m.name, first_def, m.name_span));
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
                        session.add_error(ASTError::multi_def(u.alias, first_def, u.span));
                    }

                    else {
                        ast.uses.insert(u.alias, u);
                    }
                }
                Stmt::Enum(mut e) => {
                    e.decorators = curr_decos;
                    curr_decos = vec![];

                    if let Some(first_def) = check_already_defined(&ast, &e.name) {
                        session.add_error(ASTError::multi_def(e.name, first_def, e.name_span));
                    }

                    else {
                        e.check_unused_generics(session);

                        for def in e.to_defs(&curr_location, session) {
                            ast.defs.insert(def.name, def);
                        }
                    }
                },
            }

        }

        let mut closure_collector = ClosureCollector::new();

        ast.resolve_names(session)?;
        ast.resolve_recursive_lambdas_in_block(session, &mut closure_collector)?;
        ast.modify_closure_defs(&closure_collector.closure_to_lambda_info);
        ast.clean_up_blocks(session, &mut ())?;

        if session.is_enabled(Opt::IntraInterMod) {
            let mut local_uids = ast.get_local_uids(session);
            ast.intra_inter_mod(session, &mut local_uids)?;
        }

        session.err_if_has_error()?;

        Ok(ast)
    }

    pub fn dump(&self, session: &mut LocalParseSession) -> String {
        let mut result = Vec::with_capacity(
            self.defs.len() + self.uses.len() + self.inner_modules.len()
        );

        // there are tons of `Object(XXXXX)` in the dumped result, which are not readable
        // we should translate `Object(XXXX)` into a readable name
        // we need some kind of context that has such table: UID -> FuncDef
        // but generating such table is expensive, so we have to make sure that
        // the table is generated only when something is dumped
        // I guess this is the only place to generate the table
        // after generating the table, we can just inject that to session
        // other `dump` methods will check whether the table is initialized when they encounter `Object(XXXX)`
        // if so, they'd dump a readable name, otherwise they'd just dump `Object(XXXXX)`
        let mut uid_to_name_table = HashMap::new();

        for def in self.defs.values() {
            uid_to_name_table.insert(
                def.id,
                def.pretty_name(session),
            );
        }

        let prelude_uid_table = session.get_prelude_uid_table().clone();

        for (name, uid) in prelude_uid_table.iter() {
            uid_to_name_table.insert(
                *uid,
                format!("prelude.{}", name.to_string(session)),
            );
        }

        session.update_uid_to_name_table(uid_to_name_table);

        for module in self.inner_modules.values() {
            result.push((module.def_span, module.dump(session)));
        }

        for use_ in self.uses.values() {
            result.push((use_.span, use_.dump(session)));
        }

        for def in self.defs.values() {
            result.push((def.def_span, def.dump(session)));
        }

        result.sort_by_key(|(span, _)| *span);
        let result: Vec<String> = result.into_iter().map(|(_, content)| content).collect();

        result.join("\n\n")
    }

}

// if already defined, it returns the span of the previous definition
fn check_already_defined(ast: &AST, name: &InternedString) -> Option<Span> {
    if let Some(f) = ast.defs.get(name) {
        Some(f.name_span)
    }

    else if let Some(u) = ast.uses.get(name) {
        Some(u.span)
    }

    else if let Some(m) = ast.inner_modules.get(name) {
        Some(m.name_span)
    }

    else {
        None
    }
}
