use crate::err::ParamType;
use crate::session::{InternedString, LocalParseSession};
use crate::utils::bytes_to_string;

pub enum SodigyWarningKind {
    UnusedParam(InternedString, ParamType),
}

impl SodigyWarningKind {
    pub fn render_warning(&self, session: &LocalParseSession) -> String {
        match self {
            SodigyWarningKind::UnusedParam(name, p_type) => {
                let p_type = match p_type {
                    ParamType::FuncParam | ParamType::LambdaParam => "argument",
                    ParamType::BlockDef => "local name binding",
                };

                format!(
                    "unused {p_type}: `{}`",
                    bytes_to_string(&session.unintern_string(*name)),
                )
            },
        }
    }
}
