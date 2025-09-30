use crate::Session;
use sodigy_name_analysis::{
    NameOrigin,
    Namespace,
    NamespaceKind,
};
use sodigy_span::Span;
use sodigy_string::InternedString;
use std::collections::HashSet;

impl Session {
    pub fn find_origin_and_count_usage(&mut self, id: InternedString) -> Option<(NameOrigin, Span)> {
        let mut is_local = true;
        let mut stack_index = None;
        let mut result = None;

        for (i, namespace) in self.name_stack.iter_mut().rev().enumerate() {
            match namespace {
                Namespace::FuncArg { names, index } if is_local => match names.get_mut(&id) {
                    Some((def_span, _, count)) => {
                        result = Some((NameOrigin::FuncArg { index: *index.get(&id).unwrap() }, *def_span));
                        stack_index = Some(i);
                        *count += 1;
                        break;
                    },
                    None => {},
                },
                Namespace::FuncArg { names, .. } |
                Namespace::Block { names } => match names.get_mut(&id) {
                    Some((def_span, name_kind, count)) => {
                        if is_local {
                            result = Some((NameOrigin::Local { kind: *name_kind }, *def_span));
                        }

                        else {
                            result = Some((NameOrigin::Foreign { kind: *name_kind }, *def_span));
                        }

                        stack_index = Some(i);
                        *count += 1;
                        break;
                    },
                    None => {},
                },
                Namespace::FuncDef { .. } => {
                    is_local = false;
                },
            }
        }

        match (result, stack_index) {
            (Some(result), Some(stack_index)) => {
                for namespace in self.name_stack.iter_mut().rev().take(stack_index) {
                    if let Namespace::FuncDef { foreign_names, .. } = namespace {
                        foreign_names.insert((id, result.1));
                    }
                }

                Some(result)
            },
            _ => None,
        }
    }
}
