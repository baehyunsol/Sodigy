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
            ValueKind::Identifier(ind) => String::from_utf8_lossy(
                &session
                    .unintern_string(*ind)
                    .expect("Internal Compiler Error 3E90A3A"),
            )
            .to_string(),
            ValueKind::String(ind) => format!(
                "{:?}",
                String::from_utf8_lossy(
                    &session
                        .unintern_string(*ind)
                        .expect("Internal Compiler Error 52FD790")
                )
                .to_string()
            ),
            ValueKind::List(elements) => format!(
                "[{}]",
                elements
                    .iter()
                    .map(|element| element.to_string(session))
                    .collect::<Vec<String>>()
                    .join(",")
            ),
            ValueKind::Block { defs, value } => {
                let defs = defs
                    .iter()
                    .map(|(name, value)| {
                        format!(
                            "{}={};",
                            String::from_utf8_lossy(
                                &session
                                    .unintern_string(*name)
                                    .expect("Internal Compiler Error 8029687")
                            )
                            .to_string(),
                            value.to_string(session)
                        )
                    })
                    .collect::<Vec<String>>()
                    .concat();

                format!("{}{defs}{}{}", '{', value.to_string(session), '}',)
            }
        }
    }
}
