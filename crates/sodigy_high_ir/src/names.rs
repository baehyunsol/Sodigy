use sodigy_ast::ArgDef;
use sodigy_error::substr_edit_distance;
use sodigy_intern::{InternedString, InternSession};
use sodigy_lang_item::LangItemTrait;
use sodigy_parse::IdentWithSpan;
use sodigy_prelude::PRELUDES;
use sodigy_uid::Uid;
use std::collections::HashMap;

mod endec;
mod fmt;

#[derive(Copy, Clone, Eq, Hash, PartialEq)]
pub struct IdentWithOrigin(InternedString, NameOrigin);

impl IdentWithOrigin {
    pub fn new(id: InternedString, ori: NameOrigin) -> Self {
        IdentWithOrigin(id, ori)
    }

    pub fn id(&self) -> InternedString {
        self.0
    }

    pub fn origin(&self) -> &NameOrigin {
        &self.1
    }

    pub fn set_origin(&mut self, origin: NameOrigin) {
        self.1 = origin;
    }
}

#[derive(Clone, Copy, Debug, Eq, Hash, PartialEq)]
pub enum NameOrigin {
    Prelude(Uid),
    LangItem(Uid),
    FuncArg {
        index: usize,
    },
    FuncGeneric {
        index: usize,
    },
    Local {   // match arm, `if pattern`, scope
        origin: Uid,
        binding_type: NameBindingType,
        index: usize,
    },
    Global {  // top-level `let`s, `module` and `import`

        // objects defined in the same module has uids,
        // but the objects from other modules (`import`) don't have uids yet
        origin: Option<Uid>,
    },
    Captured { lambda: Uid, index: usize },  // inside closures
}

#[derive(Clone, Copy, Debug, Eq, Hash, PartialEq)]
pub enum NameBindingType {
    ScopedLet,
    FuncArg,
    FuncGeneric,
    LambdaArg,
    MatchArm,
    IfPattern,
    Import,
}

pub struct NameSpace {
    preludes: HashMap<InternedString, Uid>,

    // `let`, `import`, and `module` in the current module
    globals: HashMap<InternedString, Option<Uid>>,

    // args and generics of the current func
    func_args: Vec<IdentWithSpan>,
    func_generics: Vec<IdentWithSpan>,

    // name bindings in `match`, scope, lambda, `if pattern`, and etc
    locals: Vec<(NameBindingType, Uid, Vec<InternedString>)>,

    pub(crate) func_args_locked: bool,
}

impl NameSpace {
    pub fn new() -> Self {
        NameSpace {
            preludes: PRELUDES.clone(),
            globals: HashMap::new(),
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
        // we don't have to clear the vectors twice

        debug_assert!(self.locals.is_empty());
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

    pub fn is_func_arg_name(&self, id: InternedString) -> bool {
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

    pub fn push_globals<_T>(&mut self, globals: &HashMap<InternedString, (_T, Option<Uid>)>) {
        for (name, (_, uid)) in globals.iter() {
            let is_none = self.globals.insert(*name, *uid);

            debug_assert!(is_none.is_none());
        }
    }

    pub fn push_locals(&mut self, name_binding_type: NameBindingType, uid: Uid, locals: Vec<InternedString>) {
        self.locals.push((name_binding_type, uid, locals));
    }

    pub fn pop_locals(&mut self) {
        self.locals.pop().unwrap();
    }

    pub fn has_this_local_uid(&self, uid: Uid) -> bool {
        // `self.locals.len()` is small enough in most cases
        self.locals.iter().any(|(_, id, _)| *id == uid)
    }

    pub fn find_arg_generic_name_collision(&self) -> Result<(), [IdentWithSpan; 2]> {
        let args = self.func_args.iter().map(
            |arg| (arg.id(), arg)
        ).collect::<HashMap<_, _>>();

        for gen in self.func_generics.iter() {
            if let Some(id) = args.get(&gen.id()) {
                return Err([**id, *gen]);
            }
        }

        Ok(())
    }

    pub fn find_origin(&self, id: InternedString, interner: &mut InternSession) -> Option<NameOrigin> {
        for (binding_type, uid, names) in self.locals.iter().rev() {
            for (index, name) in names.iter().enumerate() {
                if *name == id {
                    return Some(NameOrigin::Local {
                        binding_type: *binding_type,
                        origin: *uid,
                        index,
                    });
                }
            }
        }

        if let Some(uid) = id.try_get_lang_item_uid(interner) {
            return Some(NameOrigin::LangItem(uid));
        }

        if !self.func_args_locked {
            for (index, name) in self.func_args.iter().enumerate() {
                if name.id() == id {
                    return Some(NameOrigin::FuncArg { index });
                }
            }
        }

        for (index, name) in self.func_generics.iter().enumerate() {
            if name.id() == id {
                return Some(NameOrigin::FuncGeneric { index });
            }
        }

        if let Some(uid) = self.globals.get(&id) {
            return Some(NameOrigin::Global { origin: *uid });
        }

        if let Some(uid) = self.preludes.get(&id) {
            return Some(NameOrigin::Prelude(*uid));
        }

        None
    }

    // This is VERY EXPENSIVE.
    pub fn find_similar_names(&self, id: InternedString) -> Vec<InternedString> {
        let mut session = InternSession::new();
        let id_u8 = session.unintern_string(id).to_vec();

        // distance("f", "x") = 1, but it's not a good suggestion
        // distance("foo", "goo") = 1, and it seems like a good suggestion
        // distance("f", "F") = 0, and it seems like a good suggestion
        let similarity_threshold = (id_u8.len() / 3).max(1);

        let mut result = vec![];

        for (_, _, names) in self.locals.iter().rev() {
            for name in names.iter() {
                let name_u8 = session.unintern_string(*name).to_vec();

                if substr_edit_distance(&id_u8, &name_u8) < similarity_threshold {
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
        )).chain(self.globals.keys().map(|i| *i)).chain(
            self.preludes.iter().map(|(i, _)| *i)
        ) {
            let name_u8 = session.unintern_string(name).to_vec();

            if substr_edit_distance(&id_u8, &name_u8) < similarity_threshold {
                result.push(name);
            }
        }

        result
    }
}
