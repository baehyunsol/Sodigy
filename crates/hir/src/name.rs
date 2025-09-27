use crate::Session;
use sodigy_span::Span;
use sodigy_string::InternedString;
use std::collections::{HashMap, HashSet};

pub struct Namespace {
    pub kind: NamespaceKind,
    pub names: HashMap<InternedString, Span>,
}

impl Namespace {
    pub fn new(kind: NamespaceKind, names: HashMap<InternedString, Span>) -> Self {
        Namespace { kind, names }
    }
}

pub enum NamespaceKind {
    FuncArg,
    Block,  // declarations in a block
    Local,  // anything other than those
}

#[derive(Clone, Copy, Debug)]
pub struct IdentWithOrigin {
    pub id: InternedString,
    pub span: Span,
    pub origin: NameOrigin,

    // It's used to uniquely identify the identifiers.
    pub def_span: Span,
}

#[derive(Clone, Copy, Debug)]
pub enum NameOrigin {
    // If funcs are nested, only the inner-most function counts.
    FuncArg {
        index: usize,
    },
    // Local value that's declared inside the same function (inner-most).
    Local,
    // If this identifier is not declared inside the same function, it's Foreign.
    Foreign,
}

impl Session {
    pub fn find_origin(&self, id: InternedString) -> Option<(NameOrigin, Span)> {
        match self.curr_func_args.get(&id) {
            Some((index, def_span)) => Some((NameOrigin::FuncArg { index: *index }, *def_span)),
            None => {
                // If it hasn't met `NamespaceKind::FuncArg`, it's still inside the function,
                // so all the names are local.
                let mut is_local = true;

                for namespace in self.name_stack.iter().rev() {
                    if let Some(def_span) = namespace.names.get(&id) {
                        if is_local {
                            return Some((NameOrigin::Local, *def_span));
                        }

                        else {
                            return Some((NameOrigin::Foreign, *def_span));
                        }
                    }

                    if let NamespaceKind::FuncArg = namespace.kind {
                        is_local = false;
                    }
                }

                None
            },
        }
    }

    // `foreign_names` are collected by an inner function. So, some names might be foreign
    // to the inner function but not foreign to the outer function. It collects the names that
    // are foreign to both the inner and the outer function.
    pub fn update_foreign_names(&mut self, foreign_names: &HashSet<(InternedString, Span)>) {
        for (id, def_span) in foreign_names.iter() {
            match self.find_origin(*id) {
                Some((NameOrigin::Foreign, ds)) => {
                    assert_eq!(*def_span, ds);
                    self.foreign_names.insert((*id, *def_span));
                },
                _ => {},
            }
        }
    }
}
