use super::{DUMMY_FILE_INDEX, GLOBAL_SESSION, GLOBAL_SESSION_LOCK, InternedString, KEYWORDS, KEYWORD_START, try_init_global_session};
use crate::err::{ParseError, SodigyError};
use crate::path::Path;
use crate::token::Keyword;
use crate::warning::SodigyWarning;
use sdg_fs::read_bytes;
use std::collections::HashMap;

#[derive(Default)]
pub struct LocalParseSession {
    strings: HashMap<InternedString, Vec<u8>>,
    strings_rev: HashMap<Vec<u8>, InternedString>,
    pub(crate) curr_file: u64,
    pub(crate) is_dummy: bool,

    // it's not the actual path of file system
    // it's a sodigy style path, of the namespace
    name_path: Path,

    warnings: Vec<SodigyWarning>,
    pub errors: Vec<Box<dyn SodigyError>>,

    curr_file_data: Vec<u8>,
}

impl LocalParseSession {
    pub fn new() -> Self {
        try_init_global_session();

        LocalParseSession {
            curr_file: DUMMY_FILE_INDEX,
            is_dummy: false,
            ..Self::default()
        }
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

        // it invalidates all the stuffs that are related to spans
        self.errors = vec![];
        self.warnings = vec![];
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

        // it invalidates all the stuffs that are related to spans
        self.errors = vec![];
        self.warnings = vec![];

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

    // Expensive (if it has to write to a GlobalCache)
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
        self.warnings.iter().map(|e| e.render_warning(self)).collect::<Vec<String>>().join("\n\n")
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

    pub fn render_err(&self) -> String {
        let mut errors_sorted_by_span: Vec<&Box<dyn SodigyError>> = self.errors.iter().collect();
        errors_sorted_by_span.sort_by_key(|err| err.get_first_span());

        let mut errors = errors_sorted_by_span.iter().map(
            |e| e.render_err(self)
        ).collect::<Vec<String>>();

        errors.push(format!(
            "Could not compile `{}` due to {} previous error{}.",
            self.get_file_path(self.curr_file),
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
}
