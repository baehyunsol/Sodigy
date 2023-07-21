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
    pub(crate) name_stack: Vec<(HashSet<InternedString>, NameScopeKind)>,
    preludes: HashSet<InternedString>,
}

impl NameScope {
    // Ok(None) -> valid name, no alias
    // Ok(Some(u)) -> valid name, and has alias `u`
    // Err() -> invalid name
    pub fn search_name(&self, name: InternedString) -> Result<(Option<&Use>, NameOrigin), ()> {

        // the order of the stack doesn't matter because
        // we'll search all of them in the end anyway
        for (names, name_scope_kind) in self.name_stack.iter().rev() {

            if names.contains(&name) {
                return Ok((None, name_scope_kind.into()));
            }

        }

        if let Some(u) = self.uses.get(&name) {
            Ok((Some(u), NameOrigin::SubPath))
        }

        else if self.defs.contains(&name) {
            Ok((None, NameOrigin::Local))
        }

        else if self.preludes.contains(&name) {
            Ok((None, NameOrigin::Prelude))
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
            + self.name_stack.iter().fold(0, |c, s| c + s.0.len())
        );

        for name in self.defs.iter().chain(self.preludes.iter()) {
            result.push(session.unintern_string(*name));
        }

        for name in self.uses.iter() {
            result.push(session.unintern_string(*name.0));
        }

        // It doesn't care about what kind of stack it is
        // it only finds similar names
        for (name_stack, _) in self.name_stack.iter() {
            for name in name_stack.iter() {
                result.push(session.unintern_string(*name));
            }
        }

        result
    }

    pub fn push_names<A: GetNameOfArg>(&mut self, args: &Vec<A>, kind: NameScopeKind) {
        self.name_stack.push(
            (
                args.iter().map(
                    |arg| arg.get_name_of_arg()
                ).collect(),
                kind,
            )
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

#[derive(Copy, Clone, Eq, Hash, PartialEq)]
pub enum NameOrigin {
    NotKnownYet,
    Global,   // `a` of `use a.b.c;`
    SubPath,  // `b` of `use a.b.c;`, or `a.b()`
    Local,    // `a` of `def a: _ = _;`
    Prelude,
    FuncArg(NameScopeId),
    BlockDef(NameScopeId),
}

#[derive(Clone)]
pub enum NameScopeKind {
    Block(NameScopeId),
    FuncArg(NameScopeId),
    LambdaArg(NameScopeId),
}

impl From<&NameScopeKind> for NameOrigin {
    fn from(k: &NameScopeKind) -> Self {
        match k {
            NameScopeKind::Block(id) => NameOrigin::BlockDef(*id),
            NameScopeKind::FuncArg(id) => NameOrigin::FuncArg(*id),
            NameScopeKind::LambdaArg(id) => NameOrigin::FuncArg(*id),
        }
    }
}

#[derive(Copy, Clone, Eq, Hash, PartialEq)]
pub struct NameScopeId(u128);

impl NameScopeId {

    pub fn new_rand() -> Self {
        NameScopeId(rand::random())
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
 * It also extracts lambda functions while resolving names.
 */

impl AST {

    pub(crate) fn resolve_names(&mut self, session: &mut LocalParseSession) -> Result<(), ASTError> {
        let mut name_scope = NameScope::from_ast(self);
        let mut lambda_defs = HashMap::new();

        for func in self.defs.values_mut() {
            func.resolve_names(&mut name_scope, &mut lambda_defs, session)?;
        }

        Ok(())
    }

}