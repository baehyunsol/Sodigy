use super::ErrorKind;
use sodigy_name_analysis::NameKind;

impl ErrorKind {
    pub fn render(&self) -> String {
        match self {
            ErrorKind::UnusedName { name, kind } => {
                let name = String::from_utf8_lossy(&name.try_unintern_short_string().unwrap_or(b"???".to_vec())).to_string();
                let kind = match kind {
                    NameKind::Let => "value",
                    NameKind::Func => "function",
                    NameKind::FuncArg => "argument",
                    NameKind::Use => "import",
                    _ => panic!("TODO: {kind:?}"),
                };

                format!("unused {kind}: `{name}`")
            },
            _ => format!("{self:?}"),  // TODO
        }
    }
}
