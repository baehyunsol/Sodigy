use crate::CompilerOption;
use crate::error::ClapError;
use crate::warn::ClapWarning;

pub struct ClapSession {
    pub errors: Vec<ClapError>,
    pub warnings: Vec<ClapWarning>,
    pub result: CompilerOption,
}

impl ClapSession {
    pub fn with_result(result: CompilerOption) -> Self {
        ClapSession {
            errors: vec![],
            warnings: vec![],
            result,
        }
    }

    pub fn with_errors(errors: Vec<ClapError>) -> Self {
        ClapSession {
            errors,
            warnings: vec![],
            result: CompilerOption::default(),
        }
    }
}
