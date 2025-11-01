use sodigy_error::{Error, Warning};
use sodigy_session::Session as SodigySession;

pub struct Session {
    pub intermediate_dir: String,
    pub errors: Vec<Error>,
    pub warnings: Vec<Warning>,
}

impl Session {
    pub fn new(intermediate_dir: &str) -> Session {
        Session {
            intermediate_dir: intermediate_dir.to_string(),
            errors: vec![],
            warnings: vec![],
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
