use crate::Solver;
use sodigy_mir::{Session, Type};
use sodigy_span::Span;
use std::collections::hash_map::{Entry, HashMap};

impl Solver {
    pub fn monomorphize(&mut self, session: &mut Session) -> Result<(), ()> {
        // let mut generic_calls = HashMap::new();
        // let operators = _;

        // for type_var in self.type_vars.keys() {
        //     match type_var {
        //         Type::GenericInstance { call, generic } => match generic_calls.entry(*call) {
        //             Entry::Occupied(_) => {},
        //             Entry::Vacant(e) => {
        //                 e.insert(GenericCall {
        //                     call: *call,
        //                     def: *session.generic_def_span_rev.get(generic).unwrap(),
        //                 });
        //             },
        //         },
        //         _ => {},
        //     }
        // }

        // if generic_calls.is_empty() {
        //     return Ok(());
        // }

        // for generic_call in generic_calls.iter() {
        //     if let Some(_) = operators.get(&generic_call.def) {
        //         let generic_types = vec![];

        //         for generic_def in _.generic_defs.iter() {
        //             match session.generic_instances.get(&(generic_call.call, *generic_def)) {}
        //         }
        //     }

        //     // 1. if it's an operator,...
        //     //    get the list of the implementations of the operator
        //     //        -> problem is that, each implementation may have generic arguments
        //     // 2. if it's a function,...
        //     //    get the list of generic_def_spans of the function
        //     //    and check if all the generic args have the type
        //     //       if so: monomorphize it
        //     //       if not: push this to queue
        //     // 3. if it's a struct,...
        // }

        Ok(())
    }
}

#[derive(Clone, Debug)]
struct GenericCall {
    pub call: Span,
    pub def: Span,
}
