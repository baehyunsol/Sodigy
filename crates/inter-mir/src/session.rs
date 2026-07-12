use crate::{AssociatedFuncInstance, LogEntry, Monomorphization};
use crate::error::{TypeError, TypeWarning};
use sodigy_error::{Error, TypeVarInfo, Warning};
use sodigy_hir::{EnumShape, FuncShape, ItemShape, Poly, StructShape};
use sodigy_mir::{Session as MirSession, Type};
use sodigy_span::{Span, SpanId};
use sodigy_string::InternedString;
use std::collections::{HashMap, HashSet};

// When a type-variable is solved, it removes an entry in `type_var_refs`, but
// not in `type_vars`, because
// 1. We'll later use `type_vars` to distinguish what're infered types and what're annotated types.
// 2. If we don't remove entries in `type_var_refs`, cyclic type vars will cause a stack overflow.
pub struct Session {
    // Whenever `types.get(span)` returns `None`, it creates a type variable
    // and inserts the `span` to this hash map. It's later used to check
    // if all the type variables are infered.
    //
    // It collects `TypeVarInfo` of the type variable, which helps generating
    // the error messages.
    //
    // The key (`Type`) is either `Type::Var` or `Type::GenericArg`.
    // Every type variable the type-solver encountered must be in this map.
    // The value being `None` or `Some(_)`... doesn't mean much. It's just used to
    // help generating error messages. If you want to check if a variable has been
    // successfully infered, you have to check `.types` or `.generic_args`.
    pub type_vars: HashMap<Type, Option<TypeVarInfo>>,

    // If a type variable references another type variable, we have to track the relation.
    // For example, if a type of function `add` is `Type::Var(add) = Fn(Type::Var(x), Type::Var(y)) -> Int`,
    // we have to update `TypeVar(add)` when `TypeVar(x)` is updated. So, we `type_var_refs.get(x)`
    // will give you a vector with `add`.
    // If a type variable references itself, that should not be included in the Vec<Span>.
    //
    // A type var can be either `Type::Var` or `Type::GenericArg`.
    pub type_var_refs: HashMap<Type, Vec<Type>>,

    // If it infers that `Type::Var(x) = Type::Never`, it doesn't substitute
    // `x` with `Type::Never` and continues to infer `x`.
    // For example, if `x` is infered to `Type::Never` and `Type::Static(Int)`, it
    // chooses `Type::Static(Int)` because `Type::Never` is subtype of `Type::Static(Int)`.
    // But if it cannot find any more information about `x`, it has to choose `Type::Never`.
    // So, after type inference is done, if there's an un-infered type variable and the variable
    // is in this set, the type variable has `Type::Never`.
    pub maybe_never_type: HashMap<Type /* TypeVar */, Type /* Type::Never */>,

    // It collects the `origin` field of `Type::Blocked`.
    // Read `crates/mir/src/type.rs` for more information.
    pub blocked_type_vars: HashSet<Span>,

    // We might fail to infer type of name bindings in patterns, because
    // we don't solve the types of patterns (will later be done by MatchFsm).
    pub pattern_name_bindings: HashSet<Span>,

    // It does 2 things.
    // 1. It prevents the compiler from dispatching the same call (with the same dispatch) multiple times.
    // 2. If a call is dispatched, we shouldn't throw `CannotInferGeneric` error for the call.
    //    -> this happens for poly generics. You can dispatch a poly generic with partially infered types!
    pub solved_generic_args: HashSet<(Span /* call */, Span /* generic */)>,

    // mir_session has `funcs: Vec<Func>`, but sometimes we want to find a function by its def_span.
    // This is the map from def_span to index. So, it's safe to push functions to `mir_session.funcs`,
    // but you shouldn't change the order of `.funcs` or remove an element.
    pub funcs_rev: HashMap<Span, usize>,
    pub structs_rev: HashMap<Span, usize>,
    pub enums_rev: HashMap<Span, usize>,

    // `u64` is an id of a monomorphization.
    // 1. It prevents the compiler from doing the same monomorphization multiple times.
    // 2. It helps the compiler more helpful error messages if there's an error in a monomorphized function.
    pub monomorphizations: HashMap<u64, Monomorphization>,

    // When the session sees `x.y.z.unwrap()`, it remembers the span of `z` and def_span of
    // the associated function (which looks like `Span::Poly { .. }`).
    //
    // This is just a tmp storage. This storage is emptied after dispatch.
    pub associated_funcs: Vec<AssociatedFuncInstance>,

    // `Option.Some(3)` -> when the compiler sees this expression,
    // it remembers the call_span of `Some` and def_span of `Some` (not def_span of `Option`).
    pub call_to_variant_span: HashMap<Span, Span>,

    // These 2 fields are the result of the type-solver.
    pub types: HashMap<Span, Type>,
    pub generic_args: HashMap<(Span /* call */, Span /* generic */), Type>,

    // These 7 fields are in inter-hir session, but we cloned these
    // in order to update these.
    pub func_shapes: HashMap<Span, FuncShape>,
    pub struct_shapes: HashMap<Span, StructShape>,
    pub enum_shapes: HashMap<Span, EnumShape>,
    pub generic_to_def_span: HashMap<Span, Span>,
    pub variant_to_enum_span: HashMap<Span, Span>,
    pub equal_generic_params: HashMap<Span, Vec<(usize, usize)>>,
    pub polys: HashMap<Span, Poly>,

    pub span_string_map: HashMap<SpanId, InternedString>,
    pub lang_items: HashMap<String, Span>,
    pub built_in_funcs: HashSet<Span>,
    pub intermediate_dir: String,
    pub type_errors: Vec<TypeError>,
    pub type_warnings: Vec<TypeWarning>,
    pub errors: Vec<Error>,
    pub warnings: Vec<Warning>,

    // `TypeError::MultiplePolyCandidates` is special. If we encounter this error,
    // it's either 1) the user wrote wrong Sodigy code, so it's really an error or
    // 2) the user's code has no problem but we don't have enough information to
    // solve the poly-generic. It's because the inter-mir loop runs multiple times
    // and we incrementally collect type information.
    //
    // So we first keep the errors here. If this error still remains after we
    // have enough information, then that's a real error!
    pub maybe_type_errors: Vec<TypeError>,

    // The session collects log only if `cfg(feature = "log")` is enabled.
    pub log: Vec<LogEntry>,
}

impl Session {
    // This is for a tmp type-solver.
    // TODO: Is it safe to use empty `struct_shapes`?
    pub fn tmp(parent: &Session) -> Session {
        Session {
            type_vars: HashMap::new(),
            type_var_refs: HashMap::new(),
            maybe_never_type: HashMap::new(),
            blocked_type_vars: HashSet::new(),
            pattern_name_bindings: HashSet::new(),
            solved_generic_args: HashSet::new(),
            funcs_rev: HashMap::new(),
            structs_rev: HashMap::new(),
            enums_rev: HashMap::new(),
            monomorphizations: HashMap::new(),
            associated_funcs: vec![],
            call_to_variant_span: HashMap::new(),
            types: HashMap::new(),
            generic_args: HashMap::new(),
            func_shapes: HashMap::new(),
            struct_shapes: HashMap::new(),
            enum_shapes: HashMap::new(),
            generic_to_def_span: HashMap::new(),
            variant_to_enum_span: HashMap::new(),
            equal_generic_params: HashMap::new(),
            polys: HashMap::new(),
            span_string_map: HashMap::new(),
            lang_items: parent.lang_items.clone(),
            built_in_funcs: parent.built_in_funcs.clone(),
            intermediate_dir: parent.intermediate_dir.to_string(),
            type_errors: vec![],
            type_warnings: vec![],
            errors: vec![],
            warnings: vec![],
            maybe_type_errors: vec![],
            log: vec![],
        }
    }

    // I'm intentionally draining fields of `mir_session` to catch ICEs.
    // The functions in the inter-mir are not supposed to read the drained
    // fields of `mir_session`.
    pub fn from_mir_session(mir_session: &mut MirSession) -> Session {
        Session {
            type_vars: HashMap::new(),
            type_var_refs: HashMap::new(),
            maybe_never_type: HashMap::new(),
            blocked_type_vars: HashSet::new(),
            pattern_name_bindings: HashSet::new(),
            solved_generic_args: HashSet::new(),
            funcs_rev: mir_session.funcs.iter().enumerate().map(|(i, func)| (func.name_span.clone(), i)).collect(),
            structs_rev: mir_session.structs.iter().enumerate().map(|(i, r#struct)| (r#struct.name_span.clone(), i)).collect(),
            enums_rev: mir_session.enums.iter().enumerate().map(|(i, r#enum)| (r#enum.name_span.clone(), i)).collect(),
            monomorphizations: HashMap::new(),
            associated_funcs: vec![],
            call_to_variant_span: HashMap::new(),
            types: mir_session.types.drain().collect(),
            generic_args: mir_session.generic_args.drain().collect(),
            func_shapes: mir_session.global_context.func_shapes.take().unwrap().clone(),
            struct_shapes: mir_session.global_context.struct_shapes.take().unwrap().clone(),
            enum_shapes: mir_session.global_context.enum_shapes.take().unwrap().clone(),
            generic_to_def_span: mir_session.global_context.generic_to_def_span.take().unwrap().clone(),
            variant_to_enum_span: mir_session.global_context.variant_to_enum_span.take().unwrap().clone(),
            equal_generic_params: mir_session.equal_generic_params.drain().collect(),
            polys: mir_session.global_context.polys.take().unwrap().clone(),
            span_string_map: HashMap::new(),
            lang_items: mir_session.global_context.lang_items.take().unwrap().clone(),
            built_in_funcs: mir_session.global_context.built_in_funcs.take().unwrap().clone(),
            intermediate_dir: mir_session.intermediate_dir.to_string(),
            type_errors: vec![],
            type_warnings: vec![],
            errors: mir_session.errors.drain(..).collect(),
            warnings: mir_session.warnings.drain(..).collect(),
            maybe_type_errors: vec![],
            log: vec![],
        }
    }

    pub fn get_lang_item_span(&self, lang_item: &str) -> Span {
        match self.lang_items.get(lang_item) {
            Some(s) => s.clone(),
            None => panic!("TODO: {lang_item:?}"),
        }
    }

    pub fn get_lang_item_span_id(&self, lang_item: &str) -> SpanId {
        self.get_lang_item_span(lang_item).id().unwrap()
    }

    pub fn get_item_shape<'s>(&'s self, def_span: &Span) -> Option<ItemShape<'s>> {
        match self.struct_shapes.get(def_span) {
            Some(s) => Some(ItemShape::Struct(s)),
            None => match self.enum_shapes.get(def_span) {
                Some(e) => Some(ItemShape::Enum(e)),
                None => None,
            },
        }
    }

    #[cfg(feature = "log")]
    pub fn last_errors(&self) -> Vec<(TypeError, Error)> {
        self.type_errors.iter().rev().take(3).rev().map(
            |e| (e.clone(), self.type_error_to_general_error(e.clone()))
        ).collect()
    }

    #[cfg(feature = "log")]
    pub fn collect_type_var_info(&self, r#type: &Type) -> HashMap<Type, Option<Type>> {
        let mut result = HashMap::new();

        for type_var in r#type.get_type_vars() {
            match type_var {
                Type::Var { ref def_span, .. } => {
                    result.insert(type_var.clone(), self.types.get(def_span).cloned());
                },
                Type::GenericArg { ref call, ref generic } => {
                    result.insert(type_var.clone(), self.generic_args.get(&(call.clone(), generic.clone())).cloned());
                },
                _ => unreachable!(),
            }
        }

        result
    }
}
