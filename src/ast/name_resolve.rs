use super::{AST, ASTError};
use crate::session::{InternedString, LocalParseSession};
use crate::stmt::{GetNameOfArg, Use};
use std::collections::{HashMap, HashSet};

// TODO: where should it belong?
#[derive(Clone)]
pub struct NameScope {
    defs: HashSet<InternedString>,
    uses: HashMap<InternedString, Use>,
    pub(crate) name_stack: Vec<HashSet<InternedString>>,
    preludes: HashSet<InternedString>,
}

impl NameScope {
    // Ok(None) -> valid name, no alias
    // Ok(Some(u)) -> valid name, and has alias `u`
    // Err() -> invalid name
    pub fn search_name(&self, name: InternedString) -> Result<Option<&Use>, ()> {

        // the order of the stack doesn't matter because
        // we'll search all of them in the end anyway
        for names in self.name_stack.iter() {

            if names.contains(&name) {
                return Ok(None);
            }

        }

        if let Some(u) = self.uses.get(&name) {
            Ok(Some(u))
        }

        else if self.defs.contains(&name) {
            Ok(None)
        }

        else if self.preludes.contains(&name) {
            Ok(None)
        }

        else {
            Err(())
        }

    }

    // rust/compiler/rustc_span/src/edit_distance.rs
    // It's okay for an error-related function to be very expensive!
    pub fn get_similar_name(&self, name: InternedString, session: &LocalParseSession) -> Vec<String> {
        todo!()
    }

    pub fn push_names<A: GetNameOfArg>(&mut self, args: &Vec<A>) {
        self.name_stack.push(
            args.iter().map(
                |arg| arg.get_name_of_arg()
            ).collect()
        );
    }
}

/*
 * Name Precedence
 *
 * 1. Name Scope (defs in block_expr, args in func, args in lambda)
 *   - Close -> Far
 * 2. `use`s and `def`s
 *   - Same names not allowed
 * 3. preludes
 *
 * When it sees `use A.B.C;`, it doesn't care whether `A` is valid or not.
 * It just assumes that everything is fine. Another checker will alert the programmer
 * if `A` is invalid. Then it halts anyway...
 *
 *
 * It also finds use of undefined names while resolving names.
 */

impl AST {

    pub fn resolve_names(&mut self) -> Result<(), ASTError> {
        let mut name_scope = self.gen_name_scope();

        for func in self.defs.values_mut() {
            func.resolve_names(&mut name_scope)?;
        }

        Ok(())
    }

    pub fn gen_name_scope(&self) -> NameScope {
        NameScope {
            defs: self.defs.keys().map(|k| *k).collect::<HashSet<InternedString>>(),
            uses: self.uses.clone(),
            name_stack: vec![],
            preludes: HashSet::new(),  // TODO
        }
    }

}