use super::{
    FieldDef,
    VariantDef,
    VariantKind,
};
use std::fmt;

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
