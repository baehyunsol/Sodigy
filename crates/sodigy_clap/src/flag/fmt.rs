use super::Flag;
use sodigy_error::RenderError;

impl RenderError for Flag {
    fn render_error(&self) -> String {
        String::from_utf8_lossy(self.long()).to_string()
    }
}
