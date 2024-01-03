use sodigy_files::FileError;
use std::collections::hash_map;
use std::hash::Hasher;
use std::string::FromUtf8Error;

type Path = String;

#[derive(Clone, Debug)]
pub struct EndecError {
    pub kind: EndecErrorKind,

    // when endec/ing a file, this field is set
    path: Option<Path>,
}

impl EndecError {
    pub fn set_path(&mut self, path: &Path) -> &mut Self {
        self.path = Some(path.to_string());

        self
    }

    pub fn eof() -> Self {
        EndecError {
            kind: EndecErrorKind::Eof,
            path: None,
        }
    }

    pub fn overflow() -> Self {
        EndecError {
            kind: EndecErrorKind::Overflow,
            path: None,
        }
    }

    pub fn invalid_char(c: u32) -> Self {
        EndecError {
            kind: EndecErrorKind::InvalidChar(c),
            path: None,
        }
    }

    pub fn invalid_enum_variant(variant_index: u8) -> Self {
        EndecError {
            kind: EndecErrorKind::InvalidEnumVariant { variant_index },
            path: None,
        }
    }

    pub fn invalid_interned_numeric() -> Self {
        EndecError {
            kind: EndecErrorKind::InvalidInternedNumeric,
            path: None,
        }
    }

    pub fn invalid_interned_string() -> Self {
        EndecError {
            kind: EndecErrorKind::InvalidInternedString,
            path: None,
        }
    }

    pub fn file_is_modified() -> Self {
        EndecError {
            kind: EndecErrorKind::FileIsModified,
            path: None,
        }
    }

    pub fn human_readable_file(generated_by: &str, path: &Path) -> Self {
        EndecError {
            kind: EndecErrorKind::HumanReadableFile { generated_by: generated_by.to_string() },
            path: Some(path.to_string()),
        }
    }

    pub fn render_error(&self) -> String {
        format!(
            // TODO: implement `sodigy --help-endec-error`
            "{}{}{}\nTry `sodigy --help-endec-error` for more information.",
            if let Some(p) = &self.path {
                format!("{p}\n")
            } else {
                String::new()
            },
            match &self.kind {
                EndecErrorKind::Eof => String::from("unexpected eof"),
                EndecErrorKind::Overflow => String::from("integer overflow"),
                EndecErrorKind::FromUtf8Error(e) => format!("invalid utf-8: {e:?}"),
                EndecErrorKind::InvalidChar(c) => format!("invalid char: '\\x{c:x}'"),
                EndecErrorKind::FileError(e) => e.render_error(),
                // if file is modified, the compiler has to construct the session from scratch, not throwing this error
                EndecErrorKind::FileIsModified => String::from("If you see this error, that's an Internal Compiler Error. Please report bug."),
                EndecErrorKind::InvalidEnumVariant { variant_index } => format!("invalid enum variant: {variant_index}"),
                EndecErrorKind::InvalidInternedString => String::from("invalid interned string"),
                EndecErrorKind::InvalidInternedNumeric => String::from("invalid interned numeric"),
                EndecErrorKind::HumanReadableFile { .. } => String::from("expected a machine-readable file, got a human-readable file"),
            },
            match &self.kind {  // extra help message
                EndecErrorKind::HumanReadableFile { generated_by } => {
                    let path = self.path.clone().unwrap();

                    format!("\n`{generated_by}` dumps a human-readable file, while `--save-ir` dumps a machine-readable one.\nThe compiler only understands files generated by `--save-ir`, but it seems like `{path}` was generated by `{generated_by}`.")
                },
                _ => String::new(),
            },
        )
    }

    pub fn hash_u64(&self) -> u64 {
        let mut hasher = hash_map::DefaultHasher::new();
        hasher.write(&self.kind.hash_u64().to_be_bytes());

        if let Some(p) = &self.path {
            hasher.write(p.as_bytes());
        }

        hasher.finish()
    }
}

#[derive(Clone, Debug)]
pub enum EndecErrorKind {
    Eof,
    Overflow,
    FromUtf8Error(FromUtf8Error),
    InvalidChar(u32),
    FileError(FileError),
    FileIsModified,
    InvalidEnumVariant { variant_index: u8 },
    InvalidInternedString,
    InvalidInternedNumeric,

    // instantiate this error only when Endec has already failed
    HumanReadableFile { generated_by: String },
}

impl EndecErrorKind {
    pub fn hash_u64(&self) -> u64 {
        match self {
            EndecErrorKind::Eof => 0,
            EndecErrorKind::Overflow => 1,
            EndecErrorKind::FromUtf8Error(_) => 2,
            EndecErrorKind::InvalidChar(c) => ((*c as u64) << 8) | 3,
            EndecErrorKind::FileError(e) => {
                let h = e.hash_u64();

                h & 0xffff_ffff_ffff_0000 | 4
            },
            EndecErrorKind::FileIsModified => 5,
            EndecErrorKind::InvalidEnumVariant { variant_index } => ((*variant_index as u64) << 8) | 6,
            EndecErrorKind::InvalidInternedString => 7,
            EndecErrorKind::InvalidInternedNumeric => 8,
            EndecErrorKind::HumanReadableFile { .. } => 9,
        }
    }
}

impl From<FromUtf8Error> for EndecError {
    fn from(e: FromUtf8Error) -> Self {
        EndecError {
            kind: EndecErrorKind::FromUtf8Error(e),
            path: None,
        }
    }
}

impl From<FileError> for EndecError {
    fn from(e: FileError) -> Self {
        EndecError {
            path: e.given_path.clone(),
            kind: EndecErrorKind::FileError(e),
        }
    }
}
