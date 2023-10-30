use sodigy_ast::{ArgDef, IdentWithSpan};
use sodigy_err::substr_edit_distance;
use sodigy_intern::{InternedString, InternSession};
use sodigy_test::sodigy_assert;
use sodigy_uid::Uid;
use std::collections::{HashMap, HashSet};

mod fmt;

#[derive(Clone, Copy, Eq, Hash, PartialEq)]
pub enum NameOrigin {
    Prelude,
    FuncArg {
        index: usize,
    },
    FuncGeneric {
        index: usize,
    },
    Local {   // match arm, `if let`, scope
        origin: Uid,
        // binding_type: NameBindingType,
    },
    Global,  // `def`, `struct`, `enum`, `module`, `use`, ...
}

pub enum NameBindingType {
    LocalScope,  // `let x = 3` in `{ ... }`
    FuncArg,
    FuncGeneric,
    MatchArm,
    IfLet,
}

pub struct NameSpace {
    preludes: HashSet<InternedString>,

    // `def`, `enum`, `struct`, `use`, and `module` in the current module
    globals: HashSet<InternedString>,

    func_args: Vec<IdentWithSpan>,
    func_generics: Vec<IdentWithSpan>,

    // name bindings in `match`, scope, lambda, `if let`, and etc
    locals: Vec<(Uid, HashSet<InternedString>)>,
}

impl NameSpace {
    pub fn new() -> Self {
        NameSpace {
            preludes: HashSet::new(),
            globals: HashSet::new(),
            func_args: vec![],
            func_generics: vec![],
            locals: vec![],
        }
    }

    pub fn enter_new_func_def(&mut self) {
        self.func_args.clear();
        self.func_generics.clear();
        self.locals.clear();
    }

    pub fn leave_func_def(&mut self) {
        // TODO: what do I do here?
        // we don't have to initialize vectors twice

        sodigy_assert!(self.locals.is_empty());
    }

    pub fn push_arg(&mut self, arg: &ArgDef) -> Result<(), [IdentWithSpan; 2]> {
        // O(n), but n is small enough
        for a in self.func_args.iter() {
            if a.id() == arg.name.id() {
                return Err([arg.name, *a]);
            }
        }

        self.func_args.push(arg.name);
        Ok(())
    }

    pub fn push_generic(&mut self, generic: &IdentWithSpan) -> Result<(), [IdentWithSpan; 2]> {
        // O(n), but n is small enough
        for gen in self.func_generics.iter() {
            if gen.id() == generic.id() {
                return Err([*generic, *gen]);
            }
        }

        self.func_generics.push(*generic);
        Ok(())
    }

    pub fn push_locals(&mut self, uid: Uid, locals: HashSet<InternedString>) {
        self.locals.push((uid, locals));
    }

    pub fn pop_locals(&mut self) {
        self.locals.pop().unwrap();
    }

    pub fn find_arg_generic_name_collision(&self) -> Result<(), [IdentWithSpan; 2]> {
        let args = self.func_args.iter().map(
            |arg| (arg.id(), arg)
        ).collect::<HashMap<_, _>>();

        for gen in self.func_generics.iter() {
            if let Some(id) = args.get(gen.id()) {
                return Err([**id, *gen]);
            }
        }

        Ok(())
    }

    pub fn find_origin(&self, id: InternedString) -> Option<NameOrigin> {
        for (uid, names) in self.locals.iter().rev() {
            if names.contains(&id) {
                return Some(NameOrigin::Local { origin: *uid });
            }
        }

        for (index, name) in self.func_args.iter().enumerate() {
            if *name.id() == id {
                return Some(NameOrigin::FuncArg { index });
            }
        }

        for (index, name) in self.func_generics.iter().enumerate() {
            if *name.id() == id {
                return Some(NameOrigin::FuncGeneric { index });
            }
        }

        if self.preludes.contains(&id) {
            return Some(NameOrigin::Prelude);
        }

        None
    }

    // This is VERY EXPENSIVE.
    pub fn find_similar_names(&self, id: InternedString) -> Vec<InternedString> {
        let mut sess = InternSession::new();
        let id_u8 = match sess.unintern_string(id) {
            Some(s) => s.to_vec(),
            _ => {
                return vec![];
            }
        };

        let mut result = vec![];

        for (_, names) in self.locals.iter().rev() {
            for name in names.iter() {
                let name_u8 = match sess.unintern_string(*name) {
                    Some(s) => s.to_vec(),
                    _ => {
                        continue;
                    }
                };

                if substr_edit_distance(&id_u8, &name_u8) < 2 {
                    result.push(*name);
                }
            }
        }

        if !result.is_empty() {
            return result;
        }

        for name in self.func_args.iter().map(
            |name| name.id()
        ).chain(self.func_generics.iter().map(
            |name| name.id()
        )).chain(self.globals.iter()).chain(
            self.preludes.iter()
        ) {
            let name_u8 = match sess.unintern_string(*name) {
                Some(s) => s.to_vec(),
                _ => {
                    continue;
                }
            };

            if substr_edit_distance(&id_u8, &name_u8) < 2 {
                result.push(*name);
            }
        }

        result
    }
}
