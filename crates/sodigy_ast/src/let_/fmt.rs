use super::{Let, LetKind};
use std::fmt;

// it does not include semi colons
impl fmt::Display for Let {
    fn fmt(&self, fmt: &mut fmt::Formatter) -> Result<(), fmt::Error> {
        let mut result = Vec::with_capacity(self.attributes.len() + 1);

        for attr in self.attributes.iter() {
            result.push(attr.to_string());
        }

        result.push(self.kind.to_string());

        write!(fmt, "{}", result.join("\n"))
    }
}

impl fmt::Display for LetKind {
    fn fmt(&self, fmt: &mut fmt::Formatter) -> Result<(), fmt::Error> {
        let result = match self {
            LetKind::Pattern(pattern, value) => format!("let pattern {pattern} = {value}"),
            LetKind::Incallable {
                name,
                generics,
                return_ty,
                return_val,
                uid: _,
            } => format!(
                "let {}{}{} = {return_val}",
                name.id(),
                if generics.is_empty() {
                    String::new()
                } else {
                    format!(
                        "<{}>",
                        generics.iter().map(
                            |generic| generic.id().to_string()
                        ).collect::<Vec<_>>().join(", "),
                    )
                },
                if let Some(ty) = return_ty {
                    format!(": {ty}")
                } else {
                    String::new()
                },
            ),
            LetKind::Callable {
                name,
                args,
                generics,
                return_ty,
                return_val,
                uid: _,
            } => format!(
                "let {}({}){}{} = {return_val}",
                name.id(),
                args.iter().map(
                    |arg| arg.to_string()
                ).collect::<Vec<_>>().join(", "),
                if generics.is_empty() {
                    String::new()
                } else {
                    format!(
                        "<{}>",
                        generics.iter().map(
                            |generic| generic.id().to_string()
                        ).collect::<Vec<_>>().join(", "),
                    )
                },
                if let Some(ty) = return_ty {
                    format!(": {ty}")
                } else {
                    String::new()
                },
            ),
            LetKind::Enum {
                name,
                generics,
                variants,
                uid: _,
            } => format!(
                "let enum {}{} = {{{}}}",
                name.id(),
                if generics.is_empty() {
                    String::new()
                } else {
                    format!(
                        "<{}>",
                        generics.iter().map(
                            |generic| generic.id().to_string()
                        ).collect::<Vec<_>>().join(", "),
                    )
                },
                variants.iter().map(
                    |variant| variant.to_string()
                ).collect::<Vec<_>>().join(", "),
            ),
            LetKind::Struct {
                name,
                generics,
                fields,
                uid: _,
            } => format!(
                "let struct {}{} = {{{}}}",
                name.id(),
                if generics.is_empty() {
                    String::new()
                } else {
                    format!(
                        "<{}>",
                        generics.iter().map(
                            |generic| generic.id().to_string()
                        ).collect::<Vec<_>>().join(", "),
                    )
                },
                fields.iter().map(
                    |field| field.to_string()
                ).collect::<Vec<_>>().join(", "),
            ),
        };

        write!(fmt, "{result}")
    }
}
