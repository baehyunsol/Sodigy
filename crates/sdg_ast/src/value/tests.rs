use super::ValueKind;
use crate::session::LocalParseSession;
use crate::stmt::ArgDef;
use crate::utils::{bytes_to_string, v32_to_string};
use crate::value::BlockDef;

impl ValueKind {
    pub fn is_list(&self) -> bool {
        match self {
            ValueKind::List(_) => true,
            _ => false,
        }
    }

    pub fn is_tuple(&self) -> bool {
        match self {
            ValueKind::Tuple(_) => true,
            _ => false,
        }
    }

    pub fn to_string(&self, session: &LocalParseSession) -> String {
        match self {
            ValueKind::Integer(n) => n.to_string(),
            ValueKind::Real(n) => n.to_string(),
            ValueKind::Identifier(ind, _) => bytes_to_string(&session.unintern_string(*ind)),
            ValueKind::String(buf) => format!(
                "{:?}",
                v32_to_string(buf)
                    .expect("Internal Compiler Error 5F6D16DDCB7: {buf:?}"),
            ),
            ValueKind::Bytes(b) => format!(
                "Bytes({})",
                b.iter().map(|b| b.to_string()).collect::<Vec<String>>().join(","),
            ),
            ValueKind::List(elements) | ValueKind::Tuple(elements) | ValueKind::Format(elements) => {
                let (name, opening, closing) = if self.is_list() {
                    ("", "[", "]")
                } else if self.is_tuple() {
                    ("Tuple", "(", ")")
                } else {
                    ("Format", "(", ")")
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
                    .map(|ArgDef { name, ty, .. }| if let Some(ty) = ty {
                            format!(
                                "{}:{},",
                                bytes_to_string(&session.unintern_string(*name)),
                                ty.to_string(session),
                            )
                        } else {
                            format!("{},", bytes_to_string(&session.unintern_string(*name)))
                        }
                    )
                    .collect::<Vec<String>>()
                    .concat();

                format!("Lambda({args}{})", value.to_string(session))
            },
            ValueKind::Block { defs, value, .. } => {
                let defs = defs
                    .iter()
                    .map(|BlockDef{ name, ty, value, .. }| {
                        format!(
                            "{}{}={};",
                            bytes_to_string(&session.unintern_string(*name)),
                            if let Some(ty) = ty {
                                format!(":{}", ty.to_string(session))
                            } else {
                                String::new()
                            },
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
