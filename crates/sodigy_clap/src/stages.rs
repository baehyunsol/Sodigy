mod fmt;

#[derive(Clone, Copy, Debug, PartialEq)]
pub enum IrStage {
    Tokens, HighIr,
}

impl IrStage {
    pub fn extension(&self) -> String {
        match self {
            IrStage::Tokens => "tokens",
            IrStage::HighIr => "hir",
        }.to_string()
    }

    pub fn try_infer_from_ext(path: &str) -> Option<IrStage> {
        let path = path.to_lowercase();

        if path.ends_with(".tokens") {
            Some(IrStage::Tokens)
        }

        else if path.ends_with(".hir") {
            Some(IrStage::HighIr)
        }

        else {
            None
        }
    }
}
