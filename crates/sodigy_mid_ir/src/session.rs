use crate::func::{
    Func,
    LocalValue,
    LocalValueKey,
    MaybeInit,
    VisitFlag,
};
use crate::error::{MirError, MirErrorKind};
use crate::expr::lower_expr;
use crate::ty::{Type, lower_ty};
use crate::warn::{MirWarning, MirWarningKind};
use sodigy_config::CompilerOption;
use sodigy_error::UniversalError;
use sodigy_high_ir::{
    self as hir,
    HirSession,
    NameBindingType,
    walker_func,
};
use sodigy_intern::{
    InternedString,
    InternSession,
};
use sodigy_lang_item::LANG_ITEM_MAP;
use sodigy_prelude as prelude;
use sodigy_session::{
    SessionOutput,
    SessionSnapshot,
    SodigySession,
};
use sodigy_uid::Uid;
use std::collections::HashMap;

mod endec;

pub struct MirSession {
    errors: Vec<MirError>,
    warnings: Vec<MirWarning>,
    interner: InternSession,
    pub func_defs: HashMap<Uid, Func>,
    struct_defs: HashMap<Uid, hir::StructInfo>,

    pub curr_lowering_func: Option<Uid>,
    // only applied to `curr_lowering_func`
    local_value_table: HashMap<LocalValueSearchKey, LocalValueKey>,

    // it's for error messages and debugging mir.
    // NOTE: since every uid is unique, this map doesn't have to be here.
    //       you can even upload it on your website, and it's still valid.
    uid_name_map: HashMap<Uid, InternedString>,

    snapshots: Vec<SessionSnapshot>,
    compiler_option: CompilerOption,
    previous_errors: Vec<UniversalError>,
    previous_warnings: Vec<UniversalError>,
}

impl MirSession {
    pub fn from_hir_session(session: &HirSession) -> Self {
        MirSession {
            errors: vec![],
            warnings: vec![],
            interner: session.get_interner_cloned(),
            func_defs: HashMap::new(),
            struct_defs: session.struct_defs.clone(),
            curr_lowering_func: None,
            local_value_table: HashMap::new(),
            uid_name_map: HashMap::new(),
            snapshots: vec![],
            compiler_option: session.get_compiler_option().clone(),
            previous_errors: session.get_all_errors(),
            previous_warnings: session.get_all_warnings(),
        }
    }

    pub fn start_lowering_func(&mut self, func: Uid) {
        assert!(self.curr_lowering_func.is_none());
        self.curr_lowering_func = Some(func);
    }

    pub fn end_lowering_func(&mut self) {
        assert!(self.curr_lowering_func.is_some());
        self.curr_lowering_func = None;
    }

    pub fn register_local_values(&mut self, func: &hir::Func) -> Result<HashMap<LocalValueKey, LocalValue>, ()> {
        let (mut local_values, mut local_value_table) = collect_local_values_in_func(func);

        if let Some(args) = &func.args {
            for hir::Arg { name, ty, .. } in args.iter() {
                let ty = match ty {
                    Some(ty) => MaybeInit::Uninit(ty.clone()),
                    None => MaybeInit::None,
                };

                // it makes sense because no local_value has been popped yet
                let key = local_values.len() as u32;

                local_value_table.insert(
                    LocalValueSearchKey::FuncArg(name.id()),
                    key,
                );

                local_values.insert(
                    key,
                    LocalValue {
                        name: *name,
                        name_binding_type: NameBindingType::FuncArg,
                        value: MaybeInit::None,
                        ty,
                        is_real: true,
                        parent_func: func.uid,
                        parent_scope: None,
                        key,
                        graph: None,
                        is_valid: true,
                        visit_flag: VisitFlag::NotVisited,
                    },
                );
            }
        }

        for generic in func.generics.iter() {
            // it makes sense because no local_value has been popped yet
            let key = local_values.len() as u32;

            local_value_table.insert(
                LocalValueSearchKey::FuncGeneric(generic.id()),
                key,
            );
            local_values.insert(
                key,
                LocalValue {
                    name: *generic,
                    name_binding_type: NameBindingType::FuncGeneric,
                    value: MaybeInit::None,
                    ty: MaybeInit::Init(Type::from_uid(prelude::TYPE.1)),
                    is_real: true,
                    parent_func: func.uid,
                    parent_scope: None,
                    key,
                    graph: None,
                    is_valid: true,
                    visit_flag: VisitFlag::NotVisited,
                },
            );
        }

        self.local_value_table = local_value_table;
        let mut has_error = false;

        for local_value in local_values.values_mut() {
            match &local_value.value {
                MaybeInit::Uninit(v) => {
                    // TODO: it's lowering the type twice
                    //       duplicate error messages are handled by the compiler, but it's still inefficient
                    let v = lower_expr(
                        v,
                        local_value.ty.try_unwrap_uninit(),
                        false,
                        self,
                    );

                    if let Ok(v) = v {
                        local_value.value = MaybeInit::Init(v);
                    } else {
                        has_error = true;
                    }
                },
                MaybeInit::None => {},
                MaybeInit::Init(_) => unreachable!(),
            }

            match &local_value.ty {
                MaybeInit::Uninit(ty) => {
                    let ty = lower_ty(
                        ty,
                        self,
                    );

                    if let Ok(ty) = ty {
                        local_value.ty = MaybeInit::Init(ty);
                    } else {
                        has_error = true;
                    }
                },
                MaybeInit::None
                | MaybeInit::Init(_) => { /* nop */ },
            }
        }

        if has_error {
            return Err(());
        }

        Ok(local_values)
    }

    pub fn get_local_value_index(&self, key: LocalValueSearchKey) -> LocalValueKey {
        // It's an internal compiler error if it fails
        *self.local_value_table.get(&key).unwrap()
    }

    pub fn curr_func_uid(&self) -> Uid {
        self.curr_lowering_func.unwrap()
    }

    pub fn get_struct_info(&self, uid: Uid) -> Option<&hir::StructInfo> {
        self.struct_defs.get(&uid)
    }

    pub fn update_uid_name_map(&mut self) {
        for (uid, func) in self.func_defs.iter() {
            self.uid_name_map.insert(
                *uid,
                func.name.id(),
            );
        }

        for (name, uid) in prelude::PRELUDES.iter() {
            self.uid_name_map.insert(
                *uid,
                *name,
            );
        }

        for (uid, name) in LANG_ITEM_MAP.iter() {
            self.uid_name_map.insert(
                *uid,
                *name,
            );
        }
    }

    pub fn uid_to_string(&mut self, u: Uid) -> String {
        match self.uid_name_map.get(&u) {
            Some(id) => {
                let id = *id;

                String::from_utf8_lossy(self.interner.unintern_string(id)).to_string()
            },
            _ => u.to_ident(),
        }
    }

    // Expensive
    pub fn dump_mir(&self) -> String {
        let mut lines = Vec::with_capacity(self.func_defs.len());
        let mut func_defs = self.func_defs.values().collect::<Vec<_>>();
        func_defs.sort_by_key(|f| *f.name.span());

        for f in func_defs.iter() {
            lines.push(f.to_string());
        }

        let mut result = lines.join("\n\n");
        result = self.prettify_uids(&result);

        result
    }

    // It's very naive and expensive: it's just a bunch of string replacements.
    // It's solely for helping debugging MIR.
    fn prettify_uids(&self, s: &String) -> String {
        let mut result = s.clone();

        for (uid, name) in self.uid_name_map.iter() {
            result = result.replace(
                &uid.to_ident(),
                &name.to_string(),
            );
        }

        result
    }
}

#[derive(Eq, Hash, PartialEq)]
pub enum LocalValueSearchKey {
    FuncArg(InternedString),
    FuncGeneric(InternedString),
    LocalValue(Uid, InternedString),
}

struct CollectLocalValueContext {
    func_uid: Uid,
    local_values: HashMap<LocalValueKey, LocalValue>,
    local_value_table: HashMap<LocalValueSearchKey, LocalValueKey>,
}

fn collect_local_values_in_func(
    f: &hir::Func,
) -> (HashMap<LocalValueKey, LocalValue>, HashMap<LocalValueSearchKey, LocalValueKey>) {
    let mut context = CollectLocalValueContext {
        func_uid: f.uid,
        local_values: HashMap::new(),
        local_value_table: HashMap::new(),
    };

    walker_func(f, &mut context, &Box::new(collect_local_values_in_func_worker));

    let CollectLocalValueContext { local_values, local_value_table, .. } = context;

    (local_values, local_value_table)
}

fn collect_local_values_in_func_worker(
    e: &hir::Expr,
    c: &mut CollectLocalValueContext,
) {
    match &e.kind {
        hir::ExprKind::Scope(hir::Scope {
            lets, uid, ..
        }) => {
            for hir::ScopedLet { name, value, ty, is_real } in lets.iter() {
                let ty = if let Some(ty) = ty {
                    MaybeInit::Uninit(ty.clone())
                } else {
                    MaybeInit::None
                };
                // it makes sense because no local_value has been popped yet
                let key = c.local_values.len() as u32;

                c.local_value_table.insert(
                    LocalValueSearchKey::LocalValue(*uid, name.id()),
                    key,
                );
                c.local_values.insert(
                    key,
                    LocalValue {
                        name: *name,
                        name_binding_type: NameBindingType::ScopedLet,
                        value: MaybeInit::Uninit(value.clone()),
                        ty,
                        is_real: *is_real,
                        parent_func: c.func_uid,
                        parent_scope: Some(*uid),
                        key,
                        graph: None,
                        is_valid: true,
                        visit_flag: VisitFlag::NotVisited,
                    },
                );
            }
        },
        hir::ExprKind::Lambda(_) => todo!(),
        hir::ExprKind::Match(_) => {
            // name bindings in match statements are lowered to scoped-lets in hir pass
        },
        _ => {},
    }
}

impl SodigySession<MirError, MirErrorKind, MirWarning, MirWarningKind, HashMap<Uid, Func>, Func> for MirSession {
    fn get_errors(&self) -> &Vec<MirError> {
        &self.errors
    }

    fn get_errors_mut(&mut self) -> &mut Vec<MirError> {
        &mut self.errors
    }

    fn get_warnings(&self) -> &Vec<MirWarning> {
        &self.warnings
    }

    fn get_warnings_mut(&mut self) -> &mut Vec<MirWarning> {
        &mut self.warnings
    }

    fn get_previous_errors(&self) -> &Vec<UniversalError> {
        &self.previous_errors
    }

    fn get_previous_errors_mut(&mut self) -> &mut Vec<UniversalError> {
        &mut self.previous_errors
    }

    fn get_previous_warnings(&self) -> &Vec<UniversalError> {
        &self.previous_warnings
    }

    fn get_previous_warnings_mut(&mut self) -> &mut Vec<UniversalError> {
        &mut self.previous_warnings
    }

    fn get_results(&self) -> &HashMap<Uid, Func> {
        &self.func_defs
    }

    fn get_results_mut(&mut self) -> &mut HashMap<Uid, Func> {
        &mut self.func_defs
    }

    fn get_interner(&mut self) -> &mut InternSession {
        &mut self.interner
    }

    fn get_interner_cloned(&self) -> InternSession {
        self.interner.clone()
    }

    fn get_snapshots_mut(&mut self) -> &mut Vec<SessionSnapshot> {
        &mut self.snapshots
    }

    fn get_compiler_option(&self) -> &CompilerOption {
        &self.compiler_option
    }
}

// don't use this. just use session.get_results_mut().insert()
impl SessionOutput<Func> for HashMap<Uid, Func> {
    fn pop(&mut self) -> Option<Func> {
        unreachable!()
    }

    fn push(&mut self, _: Func) {
        unreachable!()
    }

    fn clear(&mut self) {
        self.clear();
    }

    fn len(&self) -> usize {
        self.len()
    }
}
