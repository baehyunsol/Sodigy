use super::ErrorKind;
use sodigy_name_analysis::NameKind;
use sodigy_string::unintern_string;

impl ErrorKind {
    pub fn render(&self, intern_str_map_dir: &str) -> String {
        match self {
            ErrorKind::UnusedName { name, kind } => {
                let name = String::from_utf8_lossy(&unintern_string(*name, intern_str_map_dir).unwrap().unwrap_or(b"???".to_vec())).to_string();
                let kind = match kind {
                    NameKind::Let { .. } => "value",
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
