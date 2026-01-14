use super::{ErrorKind, NameCollisionKind};
use crate::comma_list_strs;
use sodigy_name_analysis::NameKind;

impl ErrorKind {
    pub fn render(&self, intermediate_dir: &str) -> String {
        match self {
            ErrorKind::NameCollision { name, kind } => {
                let name = name.unintern_or_default(intermediate_dir);

                match kind {
                    NameCollisionKind::Block { is_top_level: true } => format!("Top-level item `{name}` is defined multiple times."),
                    NameCollisionKind::Block { is_top_level: false } => format!("Item `{name}` is defined multiple times in a block."),
                    NameCollisionKind::Enum => format!("An enum variant `{name}` is defined multiple times."),
                    NameCollisionKind::Func { params: true, generics: true } => format!(
                        "There are parameters and generics that have the same name: `{name}`.",
                    ),
                    NameCollisionKind::Func { params: true, generics: false } => format!(
                        "Function parameter `{name}` is defined multiple times.",
                    ),
                    NameCollisionKind::Func { params: false, generics: true } => format!(
                        "Function generic parameter `{name}` is defined multiple times.",
                    ),
                    NameCollisionKind::Func { params: false, generics: false } => unreachable!(),
                    NameCollisionKind::Pattern => format!("Name `{name}` is bound multiple times in a pattern."),
                    NameCollisionKind::Struct => format!("A struct field `{name}` is defined multiple times."),
                }
            },
            ErrorKind::KeywordArgumentRepeated(keyword) => format!(
                "Keyword argument `{}` is repeated.",
                keyword.unintern_or_default(intermediate_dir),
            ),
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
