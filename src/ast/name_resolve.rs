use super::{AST, ASTError};
use crate::prelude::get_preludes;
use crate::session::{InternedString, LocalParseSession};
use crate::stmt::{GetNameOfArg, Use};
use crate::utils::{bytes_to_string, edit_distance, substr_edit_distance};
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
    // THIS FUNCTION IS VERY EXPENSIVE!!
    // It's okay for an error-related function to be very expensive!
    // But don't call this function unless the compiler encounters an error!
    pub fn get_similar_name(&self, name: InternedString, session: &LocalParseSession) -> Vec<String> {
        let name: Vec<u8> = session.unintern_string(name).into_iter().map(
            |mut c| { c.make_ascii_lowercase(); c }
        ).collect();

        if name.len() < 2 {
            return vec![];
        }

        let mut result = vec![];

        let (sub_edit_distance, self_edit_distance) = if name.len() <= 4 {
            (0, 1)
        } else if name.len() <= 8 {
            (1, 1)
        } else if name.len() <= 12 {
            (1, 2)
        } else {
            (1, 3)
        };

        for curr_name in self.all_names(session).iter() {
            let lowered_name: Vec<u8> = curr_name.iter().map(
                |c| {
                    let mut c = *c;
                    c.make_ascii_lowercase();

                    c
                }
            ).collect();

            if edit_distance(&name, &lowered_name) <= self_edit_distance
                || substr_edit_distance(&name, &lowered_name) <= sub_edit_distance {
                result.push(bytes_to_string(curr_name));
            }

        }

        result.dedup();
        result
    }

    // It must be only called by `self.get_similar_name`
    fn all_names(&self, session: &LocalParseSession) -> Vec<Vec<u8>> {
        let mut result = Vec::with_capacity(
            self.defs.len() + self.preludes.len() + self.uses.len()
            + self.name_stack.iter().fold(0, |c, s| c + s.len())
        );

        for name in self.defs.iter().chain(self.preludes.iter()) {
            result.push(session.unintern_string(*name));
        }

        for name in self.uses.iter() {
            result.push(session.unintern_string(*name.0));
        }

        for name_stack in self.name_stack.iter() {
            for name in name_stack.iter() {
                result.push(session.unintern_string(*name));
            }
        }

        result
    }

    pub fn push_names<A: GetNameOfArg>(&mut self, args: &Vec<A>) {
        self.name_stack.push(
            args.iter().map(
                |arg| arg.get_name_of_arg()
            ).collect()
        );
    }

    pub fn pop_names(&mut self) {
        self.name_stack.pop().expect("Internal Compiler Error 836C6C0");
    }

    pub fn from_ast(ast: &AST) -> Self {
        NameScope {
            defs: ast.defs.keys().map(|k| *k).collect::<HashSet<InternedString>>(),
            uses: ast.uses.clone(),
            name_stack: vec![],
            preludes: get_preludes(),
        }
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
        let mut name_scope = NameScope::from_ast(self);

        for func in self.defs.values_mut() {
            func.resolve_names(&mut name_scope)?;
        }

        Ok(())
    }

}
