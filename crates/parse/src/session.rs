use crate::Block;
use sodigy_error::{Error, Warning};
use sodigy_file::File;
use sodigy_lex::Session as LexSession;

pub struct Session {
    pub file: File,
    pub ast: Block,
    pub intermediate_dir: String,
    pub is_std: bool,
    pub errors: Vec<Error>,
    pub warnings: Vec<Warning>,
}

impl Session {
    pub fn from_lex_session(lex_session: &LexSession) -> Self {
        Session {
            file: lex_session.file,
            ast: Block::dummy(),
            is_std: lex_session.is_std,
            intermediate_dir: lex_session.intermediate_dir.to_string(),
            errors: lex_session.errors.clone(),
            warnings: lex_session.warnings.clone(),
        }
    }
}
