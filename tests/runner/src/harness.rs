use crate::{CompileAndRun, CrateTest, FuzzResult, Meta};
use serde::{Deserialize, Serialize};

#[derive(Deserialize, Serialize)]
pub struct TestHarness {
    pub meta: Meta,

    // suitees that have run
    pub suites: Vec<TestSuite>,

    pub crates: Option<Vec<CrateTest>>,
    pub compile_and_run: Option<Vec<CompileAndRun>>,
    pub fuzz: Option<Vec<FuzzResult>>,
}

#[derive(Deserialize, Serialize)]
pub enum TestSuite {
    Crates,
    CompileAndRun,
    Fuzz,
}
