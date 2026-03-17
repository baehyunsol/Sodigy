use crate::subprocess;
use serde::{Deserialize, Serialize};
use sodigy_fs_api::{
    basename,
    copy_file,
    create_dir_all,
    exists,
    join,
    join3,
    read_bytes,
    read_dir,
    remove_dir_all,
};
use std::process::{Child, Command, Stdio};
use std::time::Instant;

pub struct Fuzzer {
    started_at: Instant,
    target: FuzzTarget,
    process: Child,
    artifacts_dir: String,
    corpus_dir: String,
}

#[derive(Deserialize, Serialize)]
pub struct FuzzResult {
    pub target: FuzzTarget,
    pub elapsed_ms: u64,
    pub artifact: Option<Vec<u8>>,

    // TODO: compile the artifact with the compiler, and store the stderr of the compilation
}

#[derive(Clone, Copy, Debug, Deserialize, Eq, PartialEq, Serialize)]
pub enum FuzzTarget {
    // uses sdg files in `tests/compile-and-run/` as corpus
    Cnr,

    // starts without any corpus
    Empty,
}

impl FuzzTarget {
    pub fn name(&self) -> &'static str {
        match self {
            FuzzTarget::Cnr => "cnr",
            FuzzTarget::Empty => "empty",
        }
    }
}

impl Fuzzer {
    pub fn init(
        fuzz_dir: &str,
        cnr_dir: &str,
        fuzz_target: FuzzTarget,
        quiet: bool,
    ) -> Fuzzer {
        let artifacts_dir = join3(fuzz_dir, "artifacts", fuzz_target.name()).unwrap();
        let corpus_dir = join3(fuzz_dir, "corpus", fuzz_target.name()).unwrap();

        for dir in [&artifacts_dir, &corpus_dir] {
            if exists(dir) {
                remove_dir_all(dir).unwrap();
            }

            create_dir_all(dir).unwrap();
        }

        if fuzz_target == FuzzTarget::Cnr {
            for cnr in read_dir(cnr_dir, true).unwrap() {
                if cnr.ends_with(".sdg") {
                    copy_file(
                        &cnr,
                        &join(&corpus_dir, &basename(&cnr).unwrap()).unwrap(),
                    ).unwrap();
                }
            }
        }

        if subprocess::run(
            "cargo",
            &["install", "cargo-fuzz"],
            fuzz_dir,
            300.0,
            false,
            true,
        ).is_err() {
            panic!("Failed to install cargo-fuzz. Please make sure that `cargo` is available.");
        }

        let mut fuzzer_process = Command::new("cargo");

        fuzzer_process
            .args(&[
                "+nightly",
                "fuzz",
                "run",
                fuzz_target.name(),
                "--",
                "-timeout=5",
            ])
            .current_dir(fuzz_dir);

        if quiet {
            fuzzer_process
                .stdout(Stdio::null())
                .stderr(Stdio::null());
        }

        let fuzzer_process = fuzzer_process
            .spawn()
            .expect("Failed to spawn the fuzzer process.");

        Fuzzer {
            started_at: Instant::now(),
            target: fuzz_target,
            process: fuzzer_process,
            artifacts_dir,
            corpus_dir,
        }
    }

    pub fn try_collect(&mut self) -> Option<FuzzResult> {
        match self.process.try_wait() {
            Ok(Some(status)) => {
                println!("fuzzer exited with status {:?}", status.code());
                let artifacts = read_dir(&self.artifacts_dir, false).unwrap();
                let artifact = if artifacts.is_empty() {
                    None
                } else {
                    Some(read_bytes(&artifacts[0]).unwrap())
                };
                Some(FuzzResult {
                    target: self.target,
                    elapsed_ms: Instant::now().duration_since(self.started_at.clone()).as_millis() as u64,
                    artifact,
                })
            },
            Ok(None) => None,
            Err(e) => panic!("error attempting to wait fuzzer process: {e:?}"),
        }
    }

    pub fn collect(&mut self) -> FuzzResult {
        match self.try_collect() {
            Some(r) => r,
            None => {
                self.process.kill().unwrap();
                FuzzResult {
                    target: self.target,
                    elapsed_ms: Instant::now().duration_since(self.started_at.clone()).as_millis() as u64,
                    artifact: None,
                }
            },
        }
    }
}
