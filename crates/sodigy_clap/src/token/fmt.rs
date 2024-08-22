use super::TokenKind;
use sodigy_error::RenderError;

impl RenderError for TokenKind {
    fn render_error(&self) -> String {
        match self {
            TokenKind::None => "nothing".to_string(),
            TokenKind::Path => "<PATH>".to_string(),
            TokenKind::Library => "<NAME>=<PATH>".to_string(),
            TokenKind::String => "<STRING>".to_string(),
            TokenKind::Integer => "<INTEGER>".to_string(),
            TokenKind::Flag => "<FLAG>".to_string(),
            TokenKind::Optional(arg_kind) => format!("{} or nothing", arg_kind.render_error()),
            TokenKind::EqualSign => "\'=\'".to_string(),
        }
    }
}
