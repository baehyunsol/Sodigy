use crate::error::{HirError, HirErrorKind};
use crate::func::Func;
use crate::module::Module;
use crate::struct_::StructInfo;
use crate::warn::{HirWarning, HirWarningKind};
use sodigy_ast::AstSession;
use sodigy_config::CompilerOption;
use sodigy_error::UniversalError;
use sodigy_intern::{InternedString, InternSession};
use sodigy_parse::IdentWithSpan;
use sodigy_prelude::PRELUDES;
use sodigy_session::{
    SessionOutput,
    SessionSnapshot,
    SodigySession,
};
use sodigy_span::SpanRange;
use sodigy_uid::Uid;
use std::collections::{HashMap, HashSet};

mod endec;

pub struct HirSession {
    errors: Vec<HirError>,
    warnings: Vec<HirWarning>,
    interner: InternSession,
    pub func_defs: HashMap<Uid, Func>,

    // the key is equal to `value.struct_uid`
    pub struct_defs: HashMap<Uid, StructInfo>,

    // you can get tmp names using `.allocate_tmp_name` method
    // tmp_names from this vector is guaranteed to be unique
    // (name: InternedString, used: bool)
    tmp_names: Vec<(InternedString, bool)>,

    // `_0`, `_1`, `_2`, ...
    field_exprs: Vec<InternedString>,

    // spans are used when there's an error
    pub imported_names: Vec<IdentWithSpan>,

    // modules defined in this file
    pub(crate) modules: Vec<Module>,

    snapshots: Vec<SessionSnapshot>,
    compiler_option: CompilerOption,

    previous_errors: Vec<UniversalError>,
    previous_warnings: Vec<UniversalError>,
}

impl HirSession {
    pub fn from_ast_session(session: &AstSession) -> Self {
        let mut tmp_names = vec![];
        let mut interner = session.get_interner_cloned();

        for i in 0..4 {
            tmp_names.push((
                // prefixed `@` guarantees that the users cannot use that name
                interner.intern_string(format!("@HirSessionTmpName{i}").as_bytes().to_vec()),
                false,
            ));
        }

        HirSession {
            errors: vec![],
            warnings: vec![],
            interner,
            func_defs: HashMap::new(),
            struct_defs: HashMap::new(),
            tmp_names,
            field_exprs: vec![],
            imported_names: vec![],
            modules: vec![],
            snapshots: vec![],
            compiler_option: session.get_compiler_option().clone(),
            previous_errors: session.get_all_errors(),
            previous_warnings: session.get_all_warnings(),
        }
    }

    // linear search is fine, because tmp_names.len() is very small in most cases
    pub fn allocate_tmp_name(&mut self) -> InternedString {
        for (name, used) in self.tmp_names.iter_mut() {
            if !*used {
                *used = true;
                return *name;
            }
        }

        let new_tmp_name = self.interner.intern_string(
            format!("@HirSessionTmpName{}", self.tmp_names.len()).as_bytes().to_vec()
        );

        self.tmp_names.push((new_tmp_name, /* used */ true));
        new_tmp_name
    }

    // linear search is fine, because tmp_names.len() is very small in most cases
    pub fn free_tmp_name(&mut self, name: InternedString) {
        for (name_, used) in self.tmp_names.iter_mut() {
            if *name_ == name {
                debug_assert!(*used);
                *used = false;
                return;
            }
        }

        unreachable!()
    }

    /// `0` -> `"_0"`
    pub fn get_tuple_field_expr(&mut self, index: usize) -> InternedString {
        if index < self.field_exprs.len() {
            self.field_exprs[index]
        }

        else {
            while self.field_exprs.len() <= index {
                self.field_exprs.push(
                    self.interner.intern_string(
                        format!("_{}", self.field_exprs.len()).as_bytes().to_vec()
                    )
                );
            }

            self.field_exprs[index]
        }
    }

    // `tmp` in `let Some<T>(tmp: T): Option(T) = ...;`
    pub fn make_nth_arg_name(&mut self, index: usize, span: SpanRange) -> IdentWithSpan {
        // there's no reason to define another method :)
        IdentWithSpan::new(
            self.get_tuple_field_expr(index),
            span,
        )
    }

    pub fn get_prelude_names(&self) -> HashSet<InternedString> {
        PRELUDES.keys().map(|k| *k).collect()
    }

    // Expensive
    pub fn dump_hir(&self) -> String {
        let mut lines = Vec::with_capacity(self.func_defs.len());
        let mut func_defs = self.func_defs.values().collect::<Vec<_>>();
        func_defs.sort_by_key(|f| *f.name.span());

        for f in func_defs.iter() {
            lines.push(f.to_string());
        }

        lines.join("\n\n")
    }

    pub fn add_prefix(&mut self, s: InternedString, prefix: &str) -> InternedString {
        let s = self.unintern_string(s);
        let new_s = vec![
            prefix.as_bytes().to_vec(),
            s.to_vec(),
        ].concat();

        self.intern_string(new_s)
    }
}

impl SodigySession<HirError, HirErrorKind, HirWarning, HirWarningKind, HashMap<Uid, Func>, Func> for HirSession {
    fn get_errors(&self) -> &Vec<HirError> {
        &self.errors
    }

    fn get_errors_mut(&mut self) -> &mut Vec<HirError> {
        &mut self.errors
    }

    fn get_warnings(&self) -> &Vec<HirWarning> {
        &self.warnings
    }

    fn get_warnings_mut(&mut self) -> &mut Vec<HirWarning> {
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
