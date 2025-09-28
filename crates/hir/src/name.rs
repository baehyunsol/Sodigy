use crate::Session;
use sodigy_name_analysis::{
    NameOrigin,
    NamespaceKind,
};
use sodigy_span::Span;
use sodigy_string::InternedString;
use std::collections::HashSet;

impl Session {
    pub fn find_origin(&self, id: InternedString) -> Option<(NameOrigin, Span)> {
        match self.curr_func_args.get(&id) {
            Some((index, def_span)) => Some((NameOrigin::FuncArg { index: *index }, *def_span)),
            None => {
                // If it hasn't met `NamespaceKind::FuncArg`, it's still inside the function,
                // so all the names are local.
                let mut is_local = true;

                for namespace in self.name_stack.iter().rev() {
                    if let Some((def_span, name_kind)) = namespace.names.get(&id) {
                        if is_local {
                            return Some((NameOrigin::Local { kind: *name_kind }, *def_span));
                        }

                        else {
                            return Some((NameOrigin::Foreign { kind: *name_kind }, *def_span));
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
                Some((NameOrigin::Foreign { .. }, ds)) => {
                    assert_eq!(*def_span, ds);
                    self.foreign_names.insert((*id, *def_span));
                },
                _ => {},
            }
        }
    }
}
