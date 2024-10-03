use super::Type;
use crate::session::MirSession;
use std::fmt;

impl Type {
    // I found it very difficult to stringfy `Type` without `MirSession`, so I'm not
    // implementing SodigyError::RenderError
    pub fn render_error(&self, session: &mut MirSession) -> String {
        match self {
            Type::HasToBeInferred => todo!(),
            Type::HasToBeLowered(e) => e.render_error(session),
            Type::Simple(uid) => session.uid_to_string(*uid),
        }
    }
}

impl fmt::Display for Type {
    fn fmt(&self, fmt: &mut fmt::Formatter) -> Result<(), fmt::Error> {
        let s = match self {
            Type::HasToBeInferred => String::from("_"),
            Type::HasToBeLowered(e) => e.to_string(),
            Type::Simple(uid) => uid.to_ident(),
        };

        write!(fmt, "{s}")
    }
}
