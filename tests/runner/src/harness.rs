use crate::{CompileAndRun, CrateTest, Meta};
use serde::{Deserialize, Serialize};

#[derive(Deserialize, Serialize)]
pub struct TestHarness {
    pub meta: Meta,

    // suitees that have run
    pub suites: Vec<TestSuite>,

    pub crates: Option<Vec<CrateTest>>,
    pub compile_and_run: Option<Vec<CompileAndRun>>,
}

#[derive(Deserialize, Serialize)]
pub enum TestSuite {
    Crates,
    CompileAndRun,
}
