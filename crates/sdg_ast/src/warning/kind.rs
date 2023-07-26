use crate::err::ParamType;
use crate::session::{InternedString, LocalParseSession};

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
                    name.to_string(session),
                )
            },
        }
    }
}
