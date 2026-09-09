use sodigy_stages::{CompileStage, StageExtra};

#[derive(Clone, Debug)]
pub struct TimingsEntry {
    pub stage: CompileStage,
    pub stage_extra: Option<StageExtra>,
    pub module: Option<String>,
    pub has_error: bool,

    // It's a timestamp (microseconds) since the worker's birth.
    // Clocks between workers are not synchronized, but I don't think
    // that'd be a big deal.
    pub start: u64,
    pub end: u64,
}
