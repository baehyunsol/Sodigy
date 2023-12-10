use super::{Arg, Func};
use std::fmt;

impl fmt::Display for Func {
    fn fmt(&self, fmt: &mut fmt::Formatter) -> Result<(), fmt::Error> {
        let mut result = vec![];

        result.push(format!("# {:?}\n", self.uid));

        for attribute in self.attributes.iter() {
            result.push(format!("{attribute}\n"));
        }

        result.push(format!("let {}", self.name.id()));

        if !self.generics.is_empty() {
            result.push(format!("<{}>", self.generics.iter().map(
                |g| g.id().to_string()
            ).collect::<Vec<String>>().join(", ")));
        }

        if let Some(args) = &self.args {
            result.push(format!("({})", args.iter().map(
                |arg| arg.to_string()
            ).collect::<Vec<String>>().join(", ")));
        }

        if let Some(ty) = &self.return_ty {
            result.push(format!(": {ty}"));
        }

        result.push(format!(" = {};", self.return_val));

        write!(
            fmt,
            "{}",
            result.concat(),
        )
    }
}

impl fmt::Display for Arg {
    fn fmt(&self, fmt: &mut fmt::Formatter) -> Result<(), fmt::Error> {
        write!(
            fmt,
            "{}{}{}",
            self.name.id(),
            if self.has_question_mark { "?" } else { "" },
            if let Some(ty) = &self.ty { format!(": {ty}") } else { String::new() },
        )
    }
}
