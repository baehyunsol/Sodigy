use crate::{CompileAndRun, CrateTest, FuzzResult, Meta};
use serde::{Deserialize, Serialize};

#[derive(Clone, Deserialize, Serialize)]
pub struct TestHarness {
    pub meta: Meta,

    // suites that have run
    pub suites: Vec<TestSuite>,

    pub crates: Option<Vec<CrateTest>>,
    pub compile_and_run: Option<Vec<CompileAndRun>>,
    pub fuzz: Option<Vec<FuzzResult>>,
}

impl TestHarness {
    pub fn get_cnr_blobs(&self) -> Vec<String> {
        match &self.compile_and_run {
            Some(cnrs) => cnrs.iter().map(|cnr| cnr.hash.to_string()).collect(),
            None => vec![],
        }
    }
}

#[derive(Clone, Copy, Debug, Deserialize, Eq, Hash, PartialEq, Serialize)]
pub enum TestSuite {
    Crates,
    CompileAndRun,
    Fuzz,
}
