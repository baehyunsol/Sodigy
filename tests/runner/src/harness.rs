use crate::{CompileAndRun, CrateTest, FuzzResult, Meta};
use serde::{Deserialize, Serialize};

#[derive(Clone, Deserialize, Serialize)]
pub struct TestHarness {
    pub meta: Meta,

    // If there's a critical error (e.g. you cannot even compile the test runner),
    // the CI runner will create a json file manually and fill this field.
    pub critical_error: bool,

    // suites that have run
    pub suites: Vec<TestSuite>,

    pub crates: Option<Vec<CrateTest>>,
    pub compile_and_run: Option<Vec<CompileAndRun>>,
    pub fuzz: Option<Vec<FuzzResult>>,
}

impl TestHarness {
    pub fn has_error(&self) -> bool {
        self.critical_error ||
            self.crates.as_ref().map(|c| c.iter().any(|c| c.has_error())).unwrap_or(false) ||
            self.compile_and_run.as_ref().map(|c| c.iter().any(|c| c.has_error())).unwrap_or(false) ||
            self.fuzz.as_ref().map(|f| f.iter().any(|f| f.has_error())).unwrap_or(false)
    }

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

