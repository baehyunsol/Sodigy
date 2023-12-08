use super::{Arg, Func, FuncDeco};
use std::fmt;

impl fmt::Display for Func {
    fn fmt(&self, fmt: &mut fmt::Formatter) -> Result<(), fmt::Error> {
        let mut result = vec![];

        if let Some(doc) = self.doc {
            for line in doc.to_string().lines() {
                result.push(format!("##> {line}\n"));
            }
        }

        result.push(format!("# {:?}\n", self.uid));
        result.push(self.decorators.to_string());
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

impl fmt::Display for FuncDeco {
    fn fmt(&self, fmt: &mut fmt::Formatter) -> Result<(), fmt::Error> {
        write!(fmt, "# TODO: fmt::Display for FuncDeco\n")
    }
}
