use super::ValueKind;
use crate::session::LocalParseSession;
use crate::stmt::ArgDef;

impl ValueKind {
    pub fn is_list(&self) -> bool {
        match self {
            ValueKind::List(_) => true,
            _ => false,
        }
    }

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
            ValueKind::List(elements) | ValueKind::Tuple(elements) => {
                let (name, opening, closing) = if self.is_list() {
                    ("", "[", "]")
                } else {
                    ("Tuple", "(", ")")
                };

                format!(
                    "{name}{opening}{}{closing}",
                    elements
                        .iter()
                        .map(|element| element.to_string(session))
                        .collect::<Vec<String>>()
                        .join(",")
                )
            },
            ValueKind::Lambda(args, value) => {
                let args = args
                    .iter()
                    .map(|box ArgDef { name, ty }| {
                        format!(
                            "{}:{},",
                            String::from_utf8_lossy(
                                &session
                                    .unintern_string(*name)
                                    .expect("Internal Compiler Error 5C00152")
                            )
                            .to_string(),
                            ty.to_string(session)
                        )
                    })
                    .collect::<Vec<String>>()
                    .concat();

                format!("Lambda({args}{})", value.to_string(session))
            },
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
