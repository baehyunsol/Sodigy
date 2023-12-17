use super::IrStage;
use sodigy_error::RenderError;

impl RenderError for IrStage {
    fn render_error(&self) -> String {
        match self {
            IrStage::Tokens => "tokens",
            IrStage::HighIr => "high-ir",
        }.to_string()
    }
}
