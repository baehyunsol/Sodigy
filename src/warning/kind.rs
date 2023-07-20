use crate::err::ParamType;
use crate::session::InternedString;

pub enum SodigyWarningKind {
    UnusedParam(InternedString, ParamType),
}