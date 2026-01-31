use crate::{Assert, Enum, Func, Let, Struct, Type, TypeAssertion};
use sodigy_error::{Error, Warning};
use sodigy_hir::{self as hir, FuncShape, Poly, StructShape};
use sodigy_inter_hir as inter_hir;
use sodigy_span::Span;
use sodigy_string::InternedString;
use std::collections::HashMap;

mod item_map;
mod span_string_map;

#[derive(Clone, Debug)]
pub struct Session {
    pub intermediate_dir: String,
    pub func_shapes: HashMap<Span, FuncShape>,
    pub struct_shapes: HashMap<Span, StructShape>,

    // generic def span to func def span (or struct def span) map
    pub generic_def_span_rev: HashMap<Span, Span>,

    pub lets: Vec<Let>,
    pub funcs: Vec<Func>,
    pub enums: Vec<Enum>,
    pub structs: Vec<Struct>,
    pub asserts: Vec<Assert>,

    // It's already lowered, but we need this for `span_string_map`.
    pub aliases: Vec<(InternedString, Span)>,

    pub type_assertions: Vec<TypeAssertion>,

    // It's `def_span -> type_annot` map.
    // It has type information of *every* name in the code.
    // If you query a def_span of a function, it'll give you something like `Fn(Int, Int) -> Int`.
    //
    // If first collects the type annotations, then the type-infer engine will infer the
    // missing type annotations.
    // Then the type-checker will check if all the annotations are correct.
    pub types: HashMap<Span, Type>,

    // If the programmer calls a generic function, either the programmer has to
    // annotate type, or the compiler has to infer it.
    // It's also a `type_var -> type_annot` map. It works like `Session.types`, but for generic instances.
    pub generic_args: HashMap<(Span, Span), Type>,

    // We need this when we create error messages.
    // This is really expensive to initialize, so think twice before you init this.
    pub span_string_map: Option<HashMap<Span, InternedString>>,

    pub lang_items: HashMap<String, Span>,
    pub polys: HashMap<Span, Poly>,
    pub errors: Vec<Error>,
    pub warnings: Vec<Warning>,
}

impl Session {
    pub fn from_hir(
        hir_session: &hir::Session,
        inter_hir_session: &inter_hir::Session,
    ) -> Session {
        Session {
            intermediate_dir: hir_session.intermediate_dir.clone(),
            func_shapes: inter_hir_session.func_shapes.clone(),
            struct_shapes: inter_hir_session.struct_shapes.clone(),
            generic_def_span_rev: HashMap::new(),

            // will be lowered soon
            lets: vec![],
            funcs: vec![],
            structs: vec![],

            // TODO: actually lower these
            enums: hir_session.enums.clone(),

            // will be lowered soon
            asserts: vec![],
            type_assertions: vec![],

            aliases: hir_session.aliases.iter().map(|alias| (alias.name, alias.name_span)).collect(),
            types: HashMap::new(),
            generic_args: HashMap::new(),
            span_string_map: Some(HashMap::new()),
            lang_items: inter_hir_session.lang_items.clone(),
            polys: inter_hir_session.polys.clone(),
            errors: hir_session.errors.clone(),
            warnings: hir_session.warnings.clone(),
        }
    }

    pub fn get_lang_item_span(&self, lang_item: &str) -> Span {
        match self.lang_items.get(lang_item) {
            Some(s) => *s,
            None => panic!("TODO: lang_item `{lang_item}`"),
        }
    }

    pub fn merge(&mut self, mut s: Session) {
        self.generic_def_span_rev.extend(s.generic_def_span_rev.drain());
        self.lets.extend(s.lets.drain(..));
        self.funcs.extend(s.funcs.drain(..));
        self.enums.extend(s.enums.drain(..));
        self.structs.extend(s.structs.drain(..));
        self.asserts.extend(s.asserts.drain(..));
        self.aliases.extend(s.aliases.drain(..));
        self.type_assertions.extend(s.type_assertions.drain(..));
        self.types.extend(s.types.drain());
        self.generic_args.extend(s.generic_args.drain());
        self.errors.extend(s.errors.drain(..));
        self.warnings.extend(s.warnings.drain(..));
    }

    // It only dispatches `Callable::Static`. It only replaces `def_span`, not `span`.
    pub fn dispatch(&mut self, map: &HashMap<Span, Span>) {
        for r#let in self.lets.iter_mut() {
            r#let.value.dispatch(map, &self.func_shapes, &mut self.generic_args);
        }

        for func in self.funcs.iter_mut() {
            func.value.dispatch(map, &self.func_shapes, &mut self.generic_args);
        }

        for assert in self.asserts.iter_mut() {
            assert.value.dispatch(map, &self.func_shapes, &mut self.generic_args);

            if let Some(note) = &mut assert.note {
                note.dispatch(map, &self.func_shapes, &mut self.generic_args);
            }
        }
    }
}
