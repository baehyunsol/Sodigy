use super::AST;
use sdg_prelude::{get_prelude_buffs_len, get_prelude_index};
use crate::session::{InternedString, LocalParseSession};
use crate::stmt::{GetNameOfArg, Use};
use crate::utils::{bytes_to_string, edit_distance, substr_edit_distance};
use crate::warning::SodigyWarning;
use sdg_uid::UID;
use std::collections::{HashMap, HashSet};

#[derive(Clone)]
pub struct NameScope {
    defs: HashSet<InternedString>,
    uses: HashMap<InternedString, Use>,
    used_uses: HashSet<InternedString>,
    pub(crate) name_stack: Vec<(HashSet<InternedString>, NameScopeKind)>,
    preludes: HashSet<InternedString>,
}

impl NameScope {
    // Ok(None) -> valid name, no alias
    // Ok(Some(u)) -> valid name, and has alias `u`
    // Err() -> invalid name
    pub fn search_name(&mut self, name: InternedString) -> Result<(Option<&Use>, NameOrigin), ()> {

        for (names, name_scope_kind) in self.name_stack.iter().rev() {

            if names.contains(&name) {
                return Ok((None, name_scope_kind.into()));
            }

        }

        if let Some(u) = self.uses.get(&name) {
            // if a module has `enum Foo { A, B }` and `use Foo.A as X;`, it should be `NameOrigin::Local`
            // otherwise, it's `NameOrigin::Global`
            let origin = if self.defs.contains(&u.get_first_name()) {
                NameOrigin::Local
            } else {
                NameOrigin::Global
            };

            self.used_uses.insert(name);

            Ok((Some(u), origin))
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
    // don't call this function unless the compiler encounters an error!
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
        self.name_stack.pop().expect("Internal Compiler Error 4FB22BF9906");
    }

    pub fn from_ast(ast: &AST) -> Self {
        NameScope {
            defs: ast.defs.keys().map(|k| *k).collect::<HashSet<InternedString>>(),
            uses: ast.uses.clone(),
            used_uses: HashSet::new(),
            name_stack: vec![],
            preludes: get_all_preludes(),
        }
    }
}

// TODO: cache this
fn get_all_preludes() -> HashSet<InternedString> {
    (0..get_prelude_buffs_len()).map(|i| get_prelude_index(i).into()).collect()
}

#[derive(Copy, Clone, Debug, Eq, Hash, PartialEq)]
pub enum NameOrigin {
    NotKnownYet,
    Global,   // `a` of `use a.b.c;`
    SubPath,  // `b` of `use a.b.c;`, or `a.b()`
    Local,    // `a` of `def a: _ = _;`
    Prelude,
    AnonymousFunc,  // generated by the compiler
    FuncArg(UID),
    GenericArg(UID),
    BlockDef(UID),
    MatchBranch(UID, UID),
}

impl NameOrigin {
    pub fn is_made_by_compiler(&self) -> bool {
        if let NameOrigin::AnonymousFunc = self {
            true
        } else {
            false
        }
    }

    pub fn render_err(&self) -> String {
        match self {
            NameOrigin::NotKnownYet => "an unresolved name",
            NameOrigin::Global => "a name defined in another module",
            NameOrigin::Local => "a name defined within this module",
            NameOrigin::Prelude => "a name defined inside the Sodigy std lib",
            NameOrigin::SubPath => "a name in a path",
            NameOrigin::AnonymousFunc => "a temporary name of an anonymous function",
            NameOrigin::FuncArg(_) => "an argument of a function",
            NameOrigin::GenericArg(_) => "a name of a generic argument",
            NameOrigin::BlockDef(_) => "a name binding in a block expression",
            NameOrigin::MatchBranch(_, _) => "a name binding in a match expression",
        }.to_string()
    }

    pub fn is_same_kind(&self, other: &NameOrigin) -> bool {
        match (self, other) {
            (NameOrigin::NotKnownYet, NameOrigin::NotKnownYet)
            | (NameOrigin::Global, NameOrigin::Global)
            | (NameOrigin::Local, NameOrigin::Local)
            | (NameOrigin::Prelude, NameOrigin::Prelude)
            | (NameOrigin::SubPath, NameOrigin::SubPath)
            | (NameOrigin::AnonymousFunc, NameOrigin::AnonymousFunc)
            | (NameOrigin::FuncArg(_), NameOrigin::FuncArg(_))
            | (NameOrigin::GenericArg(_), NameOrigin::GenericArg(_))
            | (NameOrigin::BlockDef(_), NameOrigin::BlockDef(_))
            | (NameOrigin::MatchBranch(_, _), NameOrigin::MatchBranch(_, _)) => true,
            _ => false,
        }
    }

    pub fn dump(&self) -> String {
        match self {
            NameOrigin::NotKnownYet | NameOrigin::Global
            | NameOrigin::SubPath | NameOrigin::Local
            | NameOrigin::Prelude | NameOrigin::AnonymousFunc => format!("{self:?}"),
            NameOrigin::FuncArg(id) => format!("FuncArg(func: {})", id.to_string()),
            NameOrigin::GenericArg(id) => format!("GenericArg(func: {})", id.to_string()),
            NameOrigin::BlockDef(id) => format!("BlockDef(block: {})", id.to_string()),
            NameOrigin::MatchBranch(m_id, b_id) => format!("MatchBranch(m: {}, b: {})", m_id.to_string(), b_id.to_string()),
        }
    }
}

#[derive(Clone)]
pub enum NameScopeKind {
    Block(UID),
    FuncArg(UID),
    GenericArg(UID),
    LambdaArg(UID),
    MatchBranch(UID, UID),  // (match id, branch id)
}

impl From<&NameScopeKind> for NameOrigin {
    fn from(k: &NameScopeKind) -> Self {
        match k {
            NameScopeKind::Block(id) => NameOrigin::BlockDef(*id),
            NameScopeKind::FuncArg(id) => NameOrigin::FuncArg(*id),
            NameScopeKind::LambdaArg(id) => NameOrigin::FuncArg(*id),
            NameScopeKind::GenericArg(id) => NameOrigin::GenericArg(*id),
            NameScopeKind::MatchBranch(m_id, b_id) => NameOrigin::MatchBranch(*m_id, *b_id),
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
 * It also extracts lambda functions while resolving names.
 */

impl AST {

    // If it returns `Err(())`, the actual errors are in `session`.
    pub(crate) fn resolve_names(&mut self, session: &mut LocalParseSession) -> Result<(), ()> {
        let mut name_scope = NameScope::from_ast(self);
        let mut lambda_defs = HashMap::new();

        for func in self.defs.values_mut() {
            func.resolve_names(&mut name_scope, &mut lambda_defs, session);
        }

        for (name, def) in lambda_defs.into_iter() {
            self.defs.insert(name, def);
        }

        for (name, use_) in self.uses.iter() {
            if !name_scope.used_uses.contains(name) {
                session.add_warning(SodigyWarning::unused_use(*name, use_.span));
            }
        }

        session.err_if_has_error()
    }

}
