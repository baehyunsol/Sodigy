use crate::session::{InternedString, LocalParseSession};
use crate::span::Span;
use crate::stmt::{FuncDef, ModDef, Stmt, Use};
use std::collections::HashMap;
use std::path::PathBuf;
use std::str::FromStr;

mod endec;
mod err;
mod name_resolve;
mod transformation;
mod walker;

use transformation::ClosureCollector;

#[cfg(test)]
mod tests;

pub use err::ASTError;
pub use name_resolve::{NameOrigin, NameScope, NameScopeKind};
pub use transformation::{LocalUIDs, TransformationKind};

/// It represents a single file.\
/// It doesn't have any data from other files, meaning that\
/// it's safe to reuse previously generated AST unless the file\
/// is modified.
pub struct AST {

    /// hash value of the (file system) path of the current file
    file_no: u64,

    pub(crate) inner_modules: HashMap<InternedString, ModDef>,
    pub defs: HashMap<InternedString, FuncDef>,
    pub(crate) uses: HashMap<InternedString, Use>,
}

impl AST {
    /// if it has an error, they're in `session`, but not returned
    pub(crate) fn from_stmts(stmts: Vec<Stmt>, session: &mut LocalParseSession) -> Result<Self, ()> {
        let mut curr_decos = vec![];
        let mut ast = AST {
            file_no: session.curr_file,
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

        // TODO: `clean_up_blocks` has to be called later, after type-checking
        // ast.clean_up_blocks(session, &mut ())?;

        // TODO: always enable this
        if session.is_enabled(TransformationKind::IntraInterMod) {
            let mut local_uids = ast.get_local_uids(session);
            ast.intra_inter_mod(session, &mut local_uids)?;
        }

        ast.mark_tail_calls();

        session.err_if_has_error()?;

        Ok(ast)
    }

    /// The span is used when there's an error with the inner module (e.g: file doesn't exist)
    pub fn get_path_of_inner_modules(&self, session: &LocalParseSession) -> Vec<(String, Span)> {
        let ast_path: PathBuf = session.get_file_path(self.file_no).into();
        let sub_path = into_sub_path(&ast_path);

        self.inner_modules.iter().map(
            |(module_name, mod_def)| (join_module_path(&sub_path, &module_name.to_string(session)), mod_def.name_span)
        ).collect()
    }

    pub fn dump(&self, session: &mut LocalParseSession) -> String {
        let mut result = Vec::with_capacity(
            self.defs.len() + self.uses.len() + self.inner_modules.len()
        );

        // there are tons of `Object(XXXXX)` in the dumped result, which are not readable.
        // we should translate `Object(XXXX)` into a readable name.
        // we need some kind of context that has such table: UID -> FuncDef
        // but generating such table is expensive, so we have to make sure that
        // the table is generated only when something is dumped.
        // I guess this is the only place to generate the table.
        // after generating the table, we can just inject that to session.
        // other `dump` methods will check whether the table is initialized when they encounter `Object(XXXX)`
        // if so, they'd dump a readable name, otherwise they'd just dump `Object(XXXXX)`
        let mut uid_to_name_table = HashMap::new();

        for def in self.defs.values() {
            uid_to_name_table.insert(
                def.id,
                def.pretty_name(session),
            );
        }

        session.update_uid_to_name_table(uid_to_name_table);
        session.update_prelude_uid_table();

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

/// `./main.sdg` -> `./`\
/// `./foo.sdg` -> `./foo/`
fn into_sub_path(path: &PathBuf) -> PathBuf {
    // it assumes that `path` is always valid
    let file_name = path.file_stem().expect("Internal Compiler Error A6D794A8F15").to_str().expect(
        "Internal Compiler Error 259DA55524B"
    );
    let parent = path.parent().expect("Internal Compiler Error 36D20D5F150").to_path_buf();

    if file_name == "main" {
        parent
    } else {
        parent.join(
            // it's infallible
            PathBuf::from_str(file_name).unwrap()
            .into_os_string().into_string()
            .expect("Internal Compiler Error E5D26AFFBAC")
        )
    }
}

fn join_module_path(sub_path: &PathBuf, module_name: &str) -> String {
    // PathBuf::from_str is infallible
    let sub_module = PathBuf::from_str(&format!("{module_name}.sdg")).unwrap();

    sub_path.join(&sub_module).into_os_string().into_string().expect("Internal Compiler Error 4B12F592B01")
}
