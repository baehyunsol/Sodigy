use super::ErrorKind;
use sodigy_name_analysis::NameKind;
use sodigy_string::unintern_string;

impl ErrorKind {
    pub fn render(&self, intermediate_dir: &str) -> String {
        match self {
            ErrorKind::UnusedName { name, kind } => {
                let name = String::from_utf8_lossy(&unintern_string(*name, intermediate_dir).unwrap().unwrap_or(b"???".to_vec())).to_string();
                let kind = match kind {
                    NameKind::Let { .. } => "value",
                    NameKind::Func => "function",
                    NameKind::Struct => "struct",
                    NameKind::Enum => "enum",
                    NameKind::Alias => "type alias",
                    NameKind::Module => "module",
                    NameKind::Use => "import",
                    NameKind::FuncArg => "argument",
                    _ => panic!("TODO: {kind:?}"),
                };

                format!("unused {kind}: `{name}`")
            },
            _ => format!("{self:?}"),  // TODO
        }
    }
}
