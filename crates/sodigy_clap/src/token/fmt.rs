use super::{Token, TokenValue};
use sodigy_error::RenderError;

impl RenderError for Token {
    fn render_error(&self) -> String {
        self.value.render_error()
    }
}

impl RenderError for TokenValue {
    fn render_error(&self) -> String {
        match self {
            TokenValue::Flag(flag) => format!("--{}", flag.render_error()),
            TokenValue::Path(path) => if path.len() < 32 {
                path.to_string()
            } else {
                String::from("<PATH>")
            },
            TokenValue::RawInput(_) => String::from("<RAW-INPUT>"),
            TokenValue::Stage(stage) => stage.render_error(),
            TokenValue::Bool(b) => if *b {
                "true"
            } else {
                "false"
            }.to_string(),
            TokenValue::Int(n) => n.to_string(),
            TokenValue::None => String::new(),
        }
    }
}
