use crate::{Assert, Enum, Func, GlobalContext, Let, Struct, Type, TypeAssertion};
use sodigy_error::{Error, Warning};
use sodigy_hir::{self as hir, FuncShape};
use sodigy_inter_hir as inter_hir;
use sodigy_span::Span;
use sodigy_string::InternedString;
use std::collections::HashMap;

mod item_map;

#[derive(Clone, Debug)]
pub struct Session<'hir, 'mir> {
    pub intermediate_dir: String,

    pub lets: Vec<Let>,
    pub funcs: Vec<Func>,
    pub enums: Vec<Enum>,
    pub structs: Vec<Struct>,
    pub asserts: Vec<Assert>,

    // It's already lowered, but we need this for `span_string_map`.
    pub aliases: Vec<(InternedString, Span)>,

    pub type_assertions: Vec<TypeAssertion>,

    // It's purely for better error messages.
    // Let's say there's a function: `fn eq<T>(lhs: T, rhs: T) -> Bool`.
    // It remembers the fact that "the first argument and the second parameter of `eq` have the same type".
    // Then, whenever it finds a call to `eq`, it checks if the two arguments have the same type.
    pub equal_generic_params: HashMap<Span, Vec<(usize, usize)>>,

    // It's `def_span -> type_annot` map.
    // It has type information of *every* name in the module.
    // Type information of other modules are in `.global_context.types`.
    // If you query a def_span of a function, it'll give you something like `Fn(Int, Int) -> Int`.
    //
    // If first collects the type annotations, then the type-infer engine will infer the
    // missing type annotations.
    // Then the type-checker will check if all the annotations are correct.
    pub types: HashMap<Span, Type>,

    // If the programmer calls a generic function, either the programmer has to
    // annotate type, or the compiler has to infer it.
    // It's also a `type_var -> type_annot` map. It works like `Session.types`, but for generic instances.
    //
    // Like `types`, it only has information of generic args in the module.
    // For generic args in other modules, use `.global_context.generic_args`.
    pub generic_args: HashMap<(Span, Span), Type>,
    pub errors: Vec<Error>,
    pub warnings: Vec<Warning>,

    // Before inter-mir, this field is empty and `types` has type information of this module.
    // While inter-mir, this field is still empty and shouldn't be used.
    // After inter-mir, this field has type information of every module and `types` is empty.
    pub global_context: GlobalContext<'hir, 'mir>,
}

impl<'hir, 'mir> Session<'hir, 'mir> {
    pub fn from_hir(
        hir_session: &hir::Session,
        inter_hir_session: &'hir inter_hir::Session,
    ) -> Session<'hir, 'static> {
        Session {
            intermediate_dir: hir_session.intermediate_dir.clone(),

            // will be lowered soon
            lets: vec![],
            funcs: vec![],
            structs: vec![],

            // TODO: actually lower these
            enums: hir_session.enums.clone(),

            // will be lowered soon
            asserts: vec![],
            type_assertions: vec![],
            equal_generic_params: HashMap::new(),

            aliases: hir_session.aliases.iter().map(|alias| (alias.name, alias.name_span.clone())).collect(),
            types: HashMap::new(),
            generic_args: HashMap::new(),
            errors: hir_session.errors.clone(),
            warnings: hir_session.warnings.clone(),
            global_context: GlobalContext::from_inter_hir_session(inter_hir_session),
        }
    }

    pub fn get_lang_item_span(&self, lang_item: &str) -> Span {
        match self.global_context.lang_items.as_ref().unwrap().get(lang_item) {
            Some(s) => s.clone(),
            None => panic!("TODO: lang_item `{lang_item}`"),
        }
    }

    pub fn merge(&mut self, mut s: Session) {
        self.lets.extend(s.lets.drain(..));
        self.funcs.extend(s.funcs.drain(..));
        self.enums.extend(s.enums.drain(..));
        self.structs.extend(s.structs.drain(..));
        self.asserts.extend(s.asserts.drain(..));
        self.aliases.extend(s.aliases.drain(..));
        self.type_assertions.extend(s.type_assertions.drain(..));
        self.equal_generic_params.extend(s.equal_generic_params.drain());
        self.types.extend(s.types.drain());
        self.generic_args.extend(s.generic_args.drain());
        self.errors.extend(s.errors.drain(..));
        self.warnings.extend(s.warnings.drain(..));
    }

    // This method is called after type-checking and monomorphization are complete.
    // So, all the generics are either monomorphized or unused.
    // Also, since the session knows the def_span of all the built_in functions, it
    // doesn't need their definitions anymore.
    pub fn remove_generics_and_builtins(&mut self) {
        self.funcs = self.funcs.drain(..).filter(
            |func| !func.built_in && func.generics.is_empty()
        ).collect();

        // TODO: structs/enums
    }

    // It only dispatches `Callable::Static`. It only replaces `def_span`, not `span`.
    pub fn dispatch(
        &mut self,
        generics: &HashMap<Span, Span>,
        associated_funcs: &HashMap<Span, Span>,
        func_shapes: &HashMap<Span, FuncShape>,
        generic_args: &mut HashMap<(Span, Span), Type>,
    ) {
        for r#let in self.lets.iter_mut() {
            r#let.value.dispatch(generics, associated_funcs, func_shapes, generic_args);
        }

        for func in self.funcs.iter_mut() {
            func.value.dispatch(generics, associated_funcs, func_shapes, generic_args);
        }

        for assert in self.asserts.iter_mut() {
            assert.value.dispatch(generics, associated_funcs, func_shapes, generic_args);

            if let Some(note) = &mut assert.note {
                note.dispatch(generics, associated_funcs, func_shapes, generic_args);
            }
        }
    }
}
