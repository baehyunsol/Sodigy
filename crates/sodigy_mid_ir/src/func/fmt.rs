use super::{Func, MaybeInit};
use std::fmt;

impl fmt::Display for Func {
    fn fmt(&self, fmt: &mut fmt::Formatter) -> Result<(), fmt::Error> {
        let mut lines = vec![];
        lines.push(format!(
            "let {}: {} = {}",
            self.name.id(),
            self.return_type,
            '{',
        ));

        if !self.local_values.is_empty() {
            lines.push(String::from("# local values:"));
        }

        for (index, local_value) in self.local_values.values().enumerate() {
            if index != 0 {
                lines.push(String::new());
            }

            lines.push(format!("    # original name: {}", local_value.name.id()));
            lines.push(format!("    # binding type: {}", local_value.name_binding_type));
            lines.push(format!(
                "    let _{}{} = {};",
                local_value.key,
                match &local_value.ty {
                    MaybeInit::None => String::new(),
                    MaybeInit::Init(ty) => format!(": {ty}"),
                    MaybeInit::Uninit(_) => unreachable!(),
                },
                match &local_value.value {
                    MaybeInit::None => String::from("_"),
                    MaybeInit::Init(e) => e.to_string(),
                    MaybeInit::Uninit(_) => unreachable!(),
                },
            ));
        }

        lines.push(String::from("# return value:"));
        lines.push(format!("    {}", self.return_value));
        lines.push(String::from("}"));

        write!(fmt, "{}", lines.join("\n"))
    }
}
