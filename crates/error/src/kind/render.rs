use super::ErrorKind;
use crate::comma_list_strs;
use sodigy_name_analysis::NameKind;
use sodigy_string::unintern_string;

impl ErrorKind {
    pub fn render(&self, intermediate_dir: &str) -> String {
        match self {
            ErrorKind::UnusedNames { names, kind } => {
                let names = names.iter().map(
                    |name| String::from_utf8_lossy(&unintern_string(*name, intermediate_dir).unwrap().unwrap_or(b"???".to_vec())).to_string()
                ).collect::<Vec<_>>();
                let names = comma_list_strs(&names, "`", "`", "and");
                let kind = match kind {
                    NameKind::Let { .. } => "value",
                    NameKind::Func => "function",
                    NameKind::Struct => "struct",
                    NameKind::Enum => "enum",
                    NameKind::EnumVariant { .. } => "enum variant",
                    NameKind::Alias => "type alias",
                    NameKind::Module => "module",
                    NameKind::Use => "import",
                    NameKind::FuncArg => "argument",
                    NameKind::Generic => "generic argument",
                    NameKind::PatternNameBind => "name binding",
                };

                format!(
                    "There {} {}unused {kind}{}: {names}",
                    if names.len() == 1 { "is" } else { "are" },
                    if names.len() == 1 { "an " } else { "" },
                    if names.len() == 1 { "" } else { "s" },
                )
            },
            _ => format!("{self:?}"),  // TODO
        }
    }
}
