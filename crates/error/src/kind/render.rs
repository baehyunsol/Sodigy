use super::ErrorKind;
use crate::comma_list_strs;
use sodigy_name_analysis::NameKind;

impl ErrorKind {
    pub fn render(&self, intermediate_dir: &str) -> String {
        match self {
            ErrorKind::UnusedNames { names, kind } => {
                let names = names.iter().map(
                    |name| name.unintern_or_default(intermediate_dir)
                ).collect::<Vec<_>>();
                let names_joined = comma_list_strs(&names, "`", "`", "and");
                let kind = match kind {
                    NameKind::Let { .. } => "value",
                    NameKind::Func => "function",
                    NameKind::Struct => "struct",
                    NameKind::Enum => "enum",
                    NameKind::EnumVariant { .. } => "enum variant",
                    NameKind::Alias => "type alias",
                    NameKind::Module => "module",
                    NameKind::Use => "import",
                    NameKind::FuncParam => "function parameter",
                    NameKind::Generic => "generic parameter",
                    NameKind::PatternNameBind => "name binding",
                    NameKind::Pipeline => "piped value",
                };

                format!(
                    "There {} {}unused {kind}{}: {names_joined}",
                    if names.len() == 1 { "is" } else { "are" },
                    if names.len() == 1 { "an " } else { "" },
                    if names.len() == 1 { "" } else { "s" },
                )
            },
            _ => format!("{self:?}"),  // TODO
        }
    }
}
