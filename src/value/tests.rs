use super::{Value, ValueKind};
use crate::session::LocalParseSession;

impl Value {

    pub fn to_string(&self, session: &LocalParseSession) -> String {
        self.kind.to_string(session)
    }

}

impl ValueKind {

    pub fn to_string(&self, session: &LocalParseSession) -> String {

        match self {
            ValueKind::Integer(n) => n.to_string(),
            ValueKind::Real(n) => n.to_string(),
            ValueKind::Identifier(ind) => String::from_utf8_lossy(&session.get_string_from_index(*ind).unwrap()).to_string(),
            ValueKind::String(ind) => format!(
                "{:?}",
                String::from_utf8_lossy(&session.get_string_from_index(*ind).unwrap()).to_string()
            ),
            ValueKind::List(elements) => format!(
                "[{}]",
                elements.iter().map(
                    |element| element.to_string(session)
                ).collect::<Vec<String>>().join(",")
            )
        }

    }

}