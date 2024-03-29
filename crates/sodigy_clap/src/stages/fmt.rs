use super::IrStage;
use sodigy_error::RenderError;

impl RenderError for IrStage {
    fn render_error(&self) -> String {
        match self {
            IrStage::HighIr => "high-ir",
            IrStage::MidIr => "mid-ir",
        }.to_string()
    }
}
