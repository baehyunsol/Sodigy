use crate::Block;
use sodigy_error::{Error, Warning};
use sodigy_file::File;
use sodigy_lex::Session as LexSession;
use sodigy_session::Session as SodigySession;
use sodigy_span::Span;

pub struct Session {
    pub file: File,
    pub ast: Block,
    pub intermediate_dir: String,
    pub main_func: Option<Span>,
    pub errors: Vec<Error>,
    pub warnings: Vec<Warning>,
}

impl Session {
    pub fn from_lex_session(lex_session: &LexSession) -> Self {
        Session {
            file: lex_session.file,
            ast: Block::dummy(),
            intermediate_dir: lex_session.intermediate_dir.to_string(),
            main_func: None,
            errors: lex_session.errors.clone(),
            warnings: lex_session.warnings.clone(),
        }
    }
}

impl SodigySession for Session {
    fn get_errors(&self) -> &[Error] {
        &self.errors
    }

    fn get_warnings(&self) -> &[Warning] {
        &self.warnings
    }

    fn get_intermediate_dir(&self) -> &str {
        &self.intermediate_dir
    }
}
