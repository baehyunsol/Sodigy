use sodigy_ast::{ArgDef, IdentWithSpan};
use sodigy_err::substr_edit_distance;
use sodigy_intern::{InternedString, InternSession};
use sodigy_prelude::PRELUDES;
use sodigy_test::sodigy_assert;
use sodigy_uid::Uid;
use std::collections::{HashMap, HashSet};

mod fmt;

#[derive(Clone, Eq, Hash, PartialEq)]
pub struct IdentWithOrigin(InternedString, NameOrigin);

impl IdentWithOrigin {
    pub fn new(id: InternedString, ori: NameOrigin) -> Self {
        IdentWithOrigin(id, ori)
    }

    pub fn id(&self) -> &InternedString {
        &self.0
    }

    pub fn origin(&self) -> &NameOrigin {
        &self.1
    }

    pub fn set_origin(&mut self, origin: NameOrigin) {
        self.1 = origin;
    }
}

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

    Captured { lambda: Uid, index: usize },  // inside closures
}

pub enum NameBindingType {
    LocalScope,  // `let x = 3` in `{ ... }`
    FuncArg,
    FuncGeneric,
    LambdaArg,
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

    pub(crate) func_args_locked: bool,
}

impl NameSpace {
    pub fn new() -> Self {
        let preludes = PRELUDES.keys().map(
            |id| *id
        ).collect();

        NameSpace {
            preludes,
            globals: HashSet::new(),
            func_args: vec![],
            func_generics: vec![],
            locals: vec![],
            func_args_locked: true,
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

    pub fn iter_func_args(&self) -> Vec<(IdentWithSpan, NameOrigin)> {
        self.func_args.iter().enumerate().map(
            |(index, arg)| (*arg, NameOrigin::FuncArg { index })
        ).collect()
    }

    pub fn iter_func_generics(&self) -> Vec<(IdentWithSpan, NameOrigin)> {
        self.func_generics.iter().enumerate().map(
            |(index, generic)| (*generic, NameOrigin::FuncGeneric { index })
        ).collect()
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

    pub fn is_func_arg_name(&self, id: &InternedString) -> bool {
        self.func_args.iter().any(|ids| ids.id() == id)
    }

    // func args are not fully pushed yet
    pub fn lock_func_args(&mut self) {
        self.func_args_locked = true;
    }

    // func args are fully pushed now
    pub fn unlock_func_args(&mut self) {
        self.func_args_locked = false;
    }

    pub fn push_locals(&mut self, uid: Uid, locals: HashSet<InternedString>) {
        self.locals.push((uid, locals));
    }

    pub fn pop_locals(&mut self) {
        self.locals.pop().unwrap();
    }

    pub fn has_this_local_uid(&self, uid: Uid) -> bool {
        // `self.locals.len()` is small enough in most cases
        self.locals.iter().any(|(id, _)| *id == uid)
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

        if !self.func_args_locked {
            for (index, name) in self.func_args.iter().enumerate() {
                if *name.id() == id {
                    return Some(NameOrigin::FuncArg { index });
                }
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

        // tmp hack to deal with the borrowck
        let empty_vec = vec![];

        // it searches func_args only when it's not locked
        let func_args_iter = if self.func_args_locked {
            empty_vec.iter()
        } else {
            self.func_args.iter()
        };

        for name in func_args_iter.map(
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
