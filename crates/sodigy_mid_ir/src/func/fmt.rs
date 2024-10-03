use super::{Func, MaybeInit};
use std::fmt;

impl fmt::Display for Func {
    fn fmt(&self, fmt: &mut fmt::Formatter) -> Result<(), fmt::Error> {
        let mut lines = vec![];
        lines.push(format!("# uid: {:?}, {}", self.uid, self.uid.to_ident()));
        lines.push(format!(
            "let {}: {} = {}",
            self.name.id(),
            self.return_type,
            '{',
        ));

        if !self.local_values.is_empty() {
            lines.push(String::from("# local values:"));
        }

        let mut local_values_sorted: Vec<_> = self.local_values.values().collect();
        local_values_sorted.sort_by_key(|lv| lv.key);

        for (index, local_value) in local_values_sorted.iter().enumerate() {
            if index != 0 {
                lines.push(String::new());
            }

            lines.push(format!("    # original name: {}", local_value.name.id()));
            lines.push(format!("    # binding type: {}", local_value.name_binding_type));
            lines.push(format!(
                "    let _{}{}{};",
                local_value.key,
                match &local_value.ty {
                    MaybeInit::None => String::new(),
                    MaybeInit::Init(ty) => format!(": {ty}"),
                    MaybeInit::Uninit(_) => unreachable!(),
                },
                match &local_value.value {
                    MaybeInit::None => String::new(),
                    MaybeInit::Init(e) => format!(" = {e}"),
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
