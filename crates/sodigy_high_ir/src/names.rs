use sodigy_intern::InternedString;
use sodigy_uid::Uid;
use std::collections::HashSet;

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
    },
    Global,  // `def`, `struct`, `enum`, `module`, `use`, ...
}

pub struct NameSpace {
    preludes: HashSet<InternedString>,

    // `def`, `enum`, `struct`, `use`, and `module` in the current module
    globals: HashSet<InternedString>,

    func_args: Vec<InternedString>,
    func_generics: Vec<InternedString>,

    // name bindings in `match`, scope, lambda, `if let`, and etc
    locals: Vec<(Uid, HashSet<InternedString>)>,
}

impl NameSpace {
    pub fn find_origin(&self, id: InternedString) -> Option<NameOrigin> {
        for (uid, names) in self.locals.iter().rev() {
            if names.contains(&id) {
                return Some(NameOrigin::Local { origin: *uid });
            }
        }

        for (index, name) in self.func_args.iter().enumerate() {
            if *name == id {
                return Some(NameOrigin::FuncArg { index });
            }
        }

        for (index, name) in self.func_generics.iter().enumerate() {
            if *name == id {
                return Some(NameOrigin::FuncGeneric { index });
            }
        }

        if self.preludes.contains(&id) {
            return Some(NameOrigin::Prelude);
        }

        None
    }
}
