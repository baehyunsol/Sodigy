use super::{
    Attribute,
    Decorator,
    FieldDef,
    VariantDef,
    VariantKind,
};
use std::fmt;

// this does not include newline characters
impl fmt::Display for Attribute {
    fn fmt(&self, fmt: &mut fmt::Formatter) -> Result<(), fmt::Error> {
        match self {
            Attribute::DocComment(d) => write!(fmt, "#> {}", d.id()),
            Attribute::Decorator(d) => write!(fmt, "{d}"),
        }
    }
}

impl fmt::Display for Decorator {
    fn fmt(&self, fmt: &mut fmt::Formatter) -> Result<(), fmt::Error> {
        write!(
            fmt, "@{}{}",
            self.name.iter().map(
                |name| name.id().to_string()
            ).collect::<Vec<_>>().join("."),
            if let Some(args) = &self.args {
                format!(
                    "({})",
                    args.iter().map(
                        |arg| arg.to_string()
                    ).collect::<Vec<_>>().join(", "),
                )
            } else {
                String::new()
            },
        )
    }
}

impl fmt::Display for VariantDef {
    fn fmt(&self, fmt: &mut fmt::Formatter) -> Result<(), fmt::Error> {
        let mut result = Vec::with_capacity(self.attributes.len() + 1);

        for attr in self.attributes.iter() {
            result.push(attr.to_string());
        }

        result.push(format!(
            "{}{}",
            self.name.id(),
            self.args,
        ));

        write!(fmt, "{}", result.join("\n"))
    }
}

impl fmt::Display for VariantKind {
    fn fmt(&self, fmt: &mut fmt::Formatter) -> Result<(), fmt::Error> {
        let res = match self {
            VariantKind::Empty => String::new(),
            VariantKind::Tuple(types) => format!(
                "({})",
                types.iter().map(
                    |ty| ty.to_string()
                ).collect::<Vec<_>>().join(", "),
            ),
            VariantKind::Struct(fields) => format!(
                "{{{}}}",
                fields.iter().map(
                    |field| field.to_string()
                ).collect::<Vec<_>>().join(", ")
            ),
        };

        write!(fmt, "{res}")
    }
}

impl fmt::Display for FieldDef {
    fn fmt(&self, fmt: &mut fmt::Formatter) -> Result<(), fmt::Error> {
        let mut result = Vec::with_capacity(self.attributes.len() + 1);

        for attr in self.attributes.iter() {
            result.push(attr.to_string());
        }

        result.push(format!("{}: {}", self.name.id(), self.ty));

        write!(fmt, "{}", result.join("\n"))
    }
}
