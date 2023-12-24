use std::string::FromUtf8Error;
use sodigy_files::FileError;

type Path = String;

#[derive(Clone, Debug)]
pub struct EndecError {
    kind: EndecErrorKind,

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

    pub fn render_error(&self) -> String {
        format!(
            // TODO: implement `sodigy --help-endec-error`
            "{}{}\nTry `sodigy --help-endec-error` for more information.",
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
                EndecErrorKind::InvalidEnumVariant { variant_index } => format!("invalid enum variant: {variant_index}"),
                EndecErrorKind::InvalidInternedString => String::from("invalid interned string"),
                EndecErrorKind::InvalidInternedNumeric => String::from("invalid interned numeric"),
            }
        )
    }
}

#[derive(Clone, Debug)]
enum EndecErrorKind {
    Eof,
    Overflow,
    FromUtf8Error(FromUtf8Error),
    InvalidChar(u32),
    FileError(FileError),
    InvalidEnumVariant { variant_index: u8 },  // TODO: is u8 big enough?
    InvalidInternedString,
    InvalidInternedNumeric,
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
