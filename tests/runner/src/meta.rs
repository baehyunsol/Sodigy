use chrono::Local;
use serde::{Deserialize, Serialize};
use std::process::Command;

pub mod git;

pub use git::CommitInfo;

#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct Meta {
    pub commit: CommitInfo,
    pub is_repo_clean: bool,
    pub cargo_version: String,
    pub rustc_version: String,
    pub os: Os,
    pub isa: Isa,
    pub started_at: String,
}

impl Meta {
    pub fn get_result_file_name(&self) -> String {
        format!(
            "sodigy-test-{}{}-{}-{}.json",
            self.commit.commit_hash.get(0..9).unwrap(),
            if self.is_repo_clean { "" } else { "-dirty" },
            format!("{:?}", self.os).to_ascii_lowercase(),
            format!("{:?}", self.isa).to_ascii_lowercase(),
        )
    }
}

#[derive(Clone, Copy, Debug, Deserialize, Eq, PartialEq, Serialize)]
pub enum Os {
    Linux,
    Mac,
    Windows,
    Etc,
}

#[derive(Clone, Copy, Debug, Deserialize, Eq, PartialEq, Serialize)]
pub enum Isa {
    // I'm just being curious... why do people call this "aarch64", not "arm64"?
    Aarch64,
    Arm32,
    X64,
    X86,
    Riscv64,
    Riscv32,
    Wasm64,
    Wasm32,
    Etc,
}

pub fn get() -> Meta {
    let started_at = Local::now().to_rfc3339();

    Meta {
        commit: git::get_commit_info(&git::get_curr_commit()),
        is_repo_clean: git::check_clean_repo(),
        cargo_version: get_cargo_version(),
        rustc_version: get_rustc_version(),
        os: get_os(),
        isa: get_isa(),
        started_at,
    }
}

fn get_os() -> Os {
    if cfg!(target_os = "linux") {
        Os::Linux
    } else if cfg!(target_os = "macos") {
        Os::Mac
    } else if cfg!(target_os = "windows") {
        Os::Windows
    } else {
        Os::Etc
    }
}

// There are a lot of target_archs and I'm not sure where I can find the exhaustive list.
// There must be more arches that are not covered.
fn get_isa() -> Isa {
    if cfg!(target_arch = "aarch64") || cfg!(target_arch = "arm64ec") {
        Isa::Aarch64
    } else if cfg!(target_arch = "arm") {
        Isa::Arm32
    } else if cfg!(target_arch = "x86_64") {
        Isa::X64
    } else if cfg!(target_arch = "x86") {
        Isa::X86
    } else if cfg!(target_arch = "riscv64") {
        Isa::Riscv64
    } else if cfg!(target_arch = "riscv32") {
        Isa::Riscv32
    } else if cfg!(target_arch = "wasm64") {
        Isa::Wasm64
    } else if cfg!(target_arch = "wasm32") {
        Isa::Wasm32
    } else {
        Isa::Etc
    }
}

fn get_cargo_version() -> String {
    let output = Command::new("cargo")
        .arg("--version")
        .output()
        .unwrap();

    String::from_utf8_lossy(&output.stdout).trim().to_string()
}

fn get_rustc_version() -> String {
    let output = Command::new("rustc")
        .arg("--version")
        .output()
        .unwrap();

    String::from_utf8_lossy(&output.stdout).trim().to_string()
}

