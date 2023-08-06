use crate::err::ParamType;
use crate::session::{InternedString, LocalParseSession};

pub enum SodigyWarningKind {
    UnusedParam(InternedString, ParamType),
    UnusedUse(InternedString),
}

impl SodigyWarningKind {
    pub fn render_warning(&self, session: &LocalParseSession) -> String {
        match self {
            SodigyWarningKind::UnusedParam(name, p_type) => {
                let p_type = match p_type {
                    ParamType::FuncParam | ParamType::LambdaParam => "argument",
                    ParamType::BlockDef => "local name binding",
                    ParamType::FuncGeneric => "generic argument",
                    ParamType::PatternNameBinding => "pattern name binding",
                    ParamType::FuncGenericAndParam => unreachable!(
                        "5E7D383F172"
                    ),
                };

                format!(
                    "unused {p_type}: `{}`",
                    name.to_string(session),
                )
            },
            SodigyWarningKind::UnusedUse(name) => format!(
                "unused import: `{}`",
                name.to_string(session),
            ),
        }
    }
}
