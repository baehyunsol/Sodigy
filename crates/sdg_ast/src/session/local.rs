use super::{DUMMY_FILE_INDEX, GLOBAL_SESSION, GLOBAL_SESSION_LOCK, InternedString, KEYWORDS, KEYWORD_START, try_init_global_session};
use crate::ast::TransformationKind;
use crate::err::{ParseError, SodigyError};
use crate::path::Path;
use crate::token::Keyword;
use crate::warning::SodigyWarning;
use sdg_fs::read_bytes;
use sdg_uid::UID;
use std::collections::HashMap;

#[derive(Default)]
pub struct LocalParseSession {
    strings: HashMap<InternedString, Vec<u8>>,
    strings_rev: HashMap<Vec<u8>, InternedString>,

    /// hash value of the (file system) path of the current file
    pub(crate) curr_file: u64,

    pub(crate) is_dummy: bool,

    /// it's not the actual path of file system\
    /// it's a sodigy style path, of the namespace
    name_path: Path,

    warnings: Vec<SodigyWarning>,
    pub errors: Vec<Box<dyn SodigyError>>,

    optimizations: HashMap<TransformationKind, bool>,

    /// it's only used for `dump` methods
    uid_to_name_table: HashMap<UID, String>,

    /// it need not initialized multiple times
    is_prelude_uid_table_init: bool,

    curr_file_data: Vec<u8>,
}

impl LocalParseSession {
    pub fn new() -> Self {
        try_init_global_session();

        let mut optimizations = HashMap::new();
        optimizations.insert(TransformationKind::IntraInterMod, true);

        let mut result = LocalParseSession {
            curr_file: DUMMY_FILE_INDEX,
            is_dummy: false,
            is_prelude_uid_table_init: false,
            optimizations,
            ..Self::default()
        };

        let root_path = Path::root(&mut result);
        result.name_path = root_path;

        result
    }

    pub fn toggle(&mut self, t: TransformationKind, flag: bool) {
        self.optimizations.insert(t, flag);
    }

    // it should have all the optimizations in the hashmap
    pub fn is_enabled(&self, t: TransformationKind) -> bool {
        *self.optimizations.get(&t).expect("Internal Compiler Error 7235E377BB9")
    }

    pub fn dummy() -> Self {
        LocalParseSession {
            curr_file: DUMMY_FILE_INDEX,
            is_dummy: true,
            ..Self::default()
        }
    }

    pub fn set_direct_input(&mut self, input: Vec<u8>) {
        self.curr_file = DUMMY_FILE_INDEX;
        self.curr_file_data = input;

        self.remove_errors_and_warnings_from_dummy();
    }

    pub fn set_input(&mut self, path: &str) -> Result<(), ParseError> {

        unsafe {
            let lock = GLOBAL_SESSION_LOCK.lock().expect("Internal Compiler Error 56241D4A08E");
            let g = GLOBAL_SESSION.as_mut().expect("Internal Compiler Error 56241D4A08E");
            self.curr_file = g.register_file(path);
            drop(lock);
        }

        match read_bytes(path) {
            Ok(b) => {
                self.curr_file_data = b;
            },
            Err(e) => {
                return Err(ParseError::file(e));
            }
        }

        Ok(())
    }

    pub fn try_unwrap_keyword(&self, index: InternedString) -> Option<Keyword> {
        let index: usize = index.into();

        if index >= KEYWORD_START as usize {
            KEYWORDS.get(index - KEYWORD_START as usize).map(|k| *k)
        }

        else {
            None
        }
    }

    /// Expensive (if it has to write to a GlobalCache)
    pub fn intern_string(&mut self, string: &[u8]) -> InternedString {
        match self.strings_rev.get(string) {
            Some(n) => *n,
            _ => {
                let result = unsafe {
                    let lock = GLOBAL_SESSION_LOCK.lock().expect("Internal Compiler Error CB9665F9D46");
                    let g = GLOBAL_SESSION.as_mut().expect("Internal Compiler Error 77C4E2EDBE9");

                    let r = g.intern_string(string);
                    drop(lock);

                    r
                };

                self.strings.insert(result, string.to_vec());
                self.strings_rev.insert(string.to_vec(), result);

                result
            }
        }
    }

    // It succeeds if `string` is already interned by this local session
    pub fn try_intern_string(&self, string: Vec<u8>) -> Option<InternedString> {
        self.strings_rev.get(&string).map(|s| *s)
    }

    pub fn unintern_string(&self, string: InternedString) -> Vec<u8> {
        match self.strings.get(&string) {
            Some(buf) => buf.to_vec(),
            None => {
                unsafe {
                    let lock = GLOBAL_SESSION_LOCK.lock().expect("Internal Compiler Error CB9665F9D46");
                    let g = GLOBAL_SESSION.as_mut().expect("Internal Compiler Error 77C4E2EDBE9");

                    let r = g.unintern_string(string);
                    drop(lock);

                    r
                }
            }
        }
    }

    pub fn add_warning(&mut self, warning: SodigyWarning) {
        self.warnings.push(warning);
    }

    pub fn add_warnings(&mut self, warnings: Vec<SodigyWarning>) {
        for warning in warnings.into_iter() {
            self.warnings.push(warning);
        }
    }

    pub fn has_no_warning(&self) -> bool {
        self.warnings.is_empty()
    }

    pub fn render_warnings(&self) -> String {
        let mut warnings: Vec<_> = self.warnings.iter().map(|w| (w.render_warning(self), w.span)).collect();
        warnings.sort_by_key(|(_, s)| *s);

        let mut warnings: Vec<_> = warnings.into_iter().map(|(m, _)| m).collect();
        warnings.push(format!(
            "this compile session generated {} warning{}",
            if warnings.len() == 1 {
                "a".to_string()
            } else {
                format!("{}", warnings.len())
            },
            if warnings.len() == 1 {
                ""
            } else {
                "s"
            }
        ));

        warnings.join("\n\n")
    }

    pub fn add_error<E: SodigyError + 'static>(&mut self, mut error: E) {
        error.try_add_more_helpful_message();
        self.errors.push(Box::new(error) as Box<dyn SodigyError>);
    }

    pub fn add_errors<E: SodigyError + 'static>(&mut self, errors: Vec<E>) {
        for mut error in errors.into_iter() {
            error.try_add_more_helpful_message();
            self.errors.push(Box::new(error) as Box<dyn SodigyError>);
        }
    }

    pub fn try_add_error<T, E: SodigyError + 'static>(&mut self, error: Result<T, E>) {
        if let Err(mut error) = error {
            error.try_add_more_helpful_message();
            self.errors.push(Box::new(error) as Box<dyn SodigyError>);
        }
    }

    pub fn has_no_error(&self) -> bool {
        self.errors.is_empty()
    }

    pub fn err_if_has_error(&self) -> Result<(), ()> {
        if self.has_no_error() {
            Ok(())
        } else {
            Err(())
        }
    }

    pub fn remove_errors_and_warnings(&mut self) {
        self.errors = vec![];
        self.warnings = vec![];
    }

    pub fn remove_errors_and_warnings_from_dummy(&mut self) {
        let mut index = 0;

        while index < self.errors.len() {
            if self.errors[index].get_first_span().file_no == DUMMY_FILE_INDEX {
                self.errors.swap_remove(index);
            } else {
                index += 1;
            }
        }

        index = 0;

        while index < self.warnings.len() {
            if self.warnings[index].span.file_no == DUMMY_FILE_INDEX {
                self.warnings.swap_remove(index);
            } else {
                index += 1;
            }
        }
    }

    pub fn render_err(&self) -> String {
        let mut errors_sorted_by_span: Vec<&Box<dyn SodigyError>> = self.errors.iter().collect();
        errors_sorted_by_span.sort_by_key(|err| err.get_first_span());

        let mut errors = errors_sorted_by_span.iter().map(
            |e| e.render_err(self)
        ).collect::<Vec<String>>();

        errors.push(format!(
            "Could not compile this session due to {} previous error{}.",
            if errors.len() == 1 {
                "a".to_string()
            } else {
                format!("{}", errors.len())
            },
            if errors.len() == 1 {
                ""
            } else {
                "s"
            }
        ));

        errors.join("\n\n")
    }

    pub(crate) fn curr_name_path(&self) -> &Path {
        &self.name_path
    }

    pub fn get_file_path(&self, index: u64) -> String {
        return unsafe {
            let lock = GLOBAL_SESSION_LOCK.lock().expect("Internal Compiler Error 9C9003FC163");
            let g = GLOBAL_SESSION.as_mut().expect("Internal Compiler Error 721788AA0BA");

            let p = g.get_file_path(index);
            drop(lock);

            p
        };
    }

    pub fn get_curr_file_content(&self) -> &[u8] {
        &self.curr_file_data
    }

    pub fn get_file_raw_content(&self, index: u64) -> Vec<u8> {
        if index == DUMMY_FILE_INDEX {
            self.curr_file_data.clone()
        }

        else {
            let path = self.get_file_path(index);

            // What do we do here? There's no way the compiler can recover from this
            read_bytes(&path).expect("Internal Compiler Error D4A59FCCCE0")
        }
    }

    pub(crate) fn get_prelude_uid_table(&self) -> &HashMap<InternedString, UID> {
        unsafe {
            let lock = GLOBAL_SESSION_LOCK.lock().expect("Internal Compiler Error 03F7671B422");
            let g = GLOBAL_SESSION.as_mut().expect("Internal Compiler Error DB0D3DEFA4B");

            let result = g.get_prelude_uid_table();

            drop(lock);

            result
        }
    }

    /// helper function for `dump` methods
    pub fn update_uid_to_name_table(&mut self, table: HashMap<UID, String>) {
        for (k, v) in table.into_iter() {
            self.uid_to_name_table.insert(k, v);
        }
    }

    pub fn update_prelude_uid_table(&mut self) {
        if self.is_prelude_uid_table_init {
            return;
        }

        let prelude_uid_table = self.get_prelude_uid_table().clone();

        for (name, uid) in prelude_uid_table.iter() {
            self.uid_to_name_table.insert(
                *uid,
                format!("prelude.{}", name.to_string(self)),
            );
        }

        self.is_prelude_uid_table_init = true;
    }

    /// helper function for `dump` methods
    pub(crate) fn get_name_from_uid(&self, uid: &UID) -> Option<String> {
        self.uid_to_name_table.get(uid).map(|s| s.to_string())
    }
}
