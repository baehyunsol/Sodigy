use super::ValueKind;
use crate::session::LocalParseSession;
use crate::stmt::ArgDef;
use crate::utils::{bytes_to_string, v32_to_string};

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
            ValueKind::Identifier(ind) => bytes_to_string(&session.unintern_string(*ind)),
            ValueKind::String(buf) => format!(
                "{:?}",
                v32_to_string(buf)
                    .expect("Internal Compiler Error 552D806: {buf:?}"),
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
                            bytes_to_string(&session.unintern_string(*name)),
                            ty.to_string(session),
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
                            bytes_to_string(&session.unintern_string(*name)),
                            value.to_string(session),
                        )
                    })
                    .collect::<Vec<String>>()
                    .concat();

                format!("{}{defs}{}{}", '{', value.to_string(session), '}',)
            }
        }
    }
}
