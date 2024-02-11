mod fmt;

#[derive(Clone, Copy, Debug, PartialEq)]
pub enum IrStage {
    HighIr, MidIr,
}

impl IrStage {
    pub fn extension(&self) -> String {
        match self {
            IrStage::HighIr => "hir",
            IrStage::MidIr => "mir",
        }.to_string()
    }

    pub fn try_infer_from_ext(path: &str) -> Option<IrStage> {
        let path = path.to_lowercase();

        if path.ends_with(".hir") {
            Some(IrStage::HighIr)
        }

        else if path.ends_with(".mir") {
            Some(IrStage::MidIr)
        }

        else {
            None
        }
    }
}
