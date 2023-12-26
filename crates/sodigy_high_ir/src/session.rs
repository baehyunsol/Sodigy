use crate::error::HirError;
use crate::func::Func;
use crate::warn::HirWarning;
use smallvec::{SmallVec, smallvec};
use sodigy_ast::IdentWithSpan;
use sodigy_intern::{InternedNumeric, InternedString, InternSession};
use sodigy_number::SodigyNumber;
use sodigy_prelude::PRELUDES;
use sodigy_span::SpanRange;
use sodigy_test::sodigy_assert;
use std::collections::{HashMap, HashSet};

mod endec;

pub struct HirSession {
    pub errors: Vec<HirError>,
    pub warnings: Vec<HirWarning>,
    interner: InternSession,

    // HashMap<name, def>
    pub func_defs: HashMap<InternedString, Func>,

    // you can get tmp names using `.allocate_tmp_name` method
    // tmp_names from this vector is guaranteed to be unique
    // (name: InternedString, used: bool)
    tmp_names: SmallVec<[(InternedString, bool); 4]>,

    // `_0`, `_1`, `_2`, ...
    field_exprs: Vec<InternedString>,
}

impl HirSession {
    pub fn new() -> Self {
        let mut interner = InternSession::new();
        let mut tmp_names = smallvec![];

        for i in 0..4 {
            tmp_names.push((
                // prefixed `@` guarantees that the users cannot use that name
                interner.intern_string(format!("@HirSessionTmpName{i}").as_bytes().to_vec()),
                false,
            ));
        }

        let field_exprs = (0..8).map(
            |i| interner.intern_string(
                format!("_{i}").as_bytes().to_vec()
            )
        ).collect();

        HirSession {
            errors: vec![],
            warnings: vec![],
            interner,
            tmp_names,
            field_exprs,
            func_defs: HashMap::new(),
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
                sodigy_assert!(*used);
                *used = false;
                return;
            }
        }

        unreachable!()
    }

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
    pub fn make_nth_arg_name(&mut self, index: usize) -> IdentWithSpan {
        // there's no reason to define another method :)
        IdentWithSpan::new(
            self.get_tuple_field_expr(index),
            SpanRange::dummy(11),
        )
    }

    pub fn get_prelude_names(&self) -> HashSet<InternedString> {
        PRELUDES.keys().map(|k| *k).collect()
    }

    // Expensive
    pub fn dump_hir(&self) -> String {
        let mut lines = Vec::with_capacity(self.func_defs.len());
        let mut func_defs = self.func_defs.values().map(|f| f.clone()).collect::<Vec<_>>();
        func_defs.sort_by_key(|f| *f.name.span());

        for f in func_defs.iter() {
            lines.push(f.to_string());
        }

        lines.join("\n\n")
    }

    pub fn push_error(&mut self, error: HirError) {
        self.errors.push(error);
    }

    pub fn get_errors(&self) -> &Vec<HirError> {
        &self.errors
    }

    pub fn push_warning(&mut self, warning: HirWarning) {
        self.warnings.push(warning);
    }

    pub fn get_warnings(&self) -> &Vec<HirWarning> {
        &self.warnings
    }

    pub fn intern_numeric(&mut self, n: SodigyNumber) -> InternedNumeric {
        self.interner.intern_numeric(n)
    }

    pub fn unintern_numeric(&mut self, s: InternedNumeric) -> Option<&SodigyNumber> {
        self.interner.unintern_numeric(s)
    }

    pub fn intern_string(&mut self, s: Vec<u8>) -> InternedString {
        self.interner.intern_string(s)
    }

    pub fn unintern_string(&mut self, s: InternedString) -> Option<&[u8]> {
        self.interner.unintern_string(s)
    }

    pub fn add_prefix(&mut self, s: InternedString, prefix: &str) -> InternedString {
        if let Some(s) = self.unintern_string(s) {
            let new_s = vec![
                prefix.as_bytes().to_vec(),
                s.to_vec(),
            ].concat();

            self.intern_string(new_s)
        }

        else {
            // TODO: if it's unreachable, what's the point of `HirSession::unintern_string`?
            // if that never returns `None`, why is the return type `Option<&[u8]>`
            unreachable!()
        }
    }

    // TODO: no more `err_if_has_err`
    pub fn err_if_has_err(&self) -> Result<(), ()> {
        if self.errors.is_empty() {
            Ok(())
        }

        else {
            Err(())
        }
    }
}
