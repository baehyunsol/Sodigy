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
    pub started_at: String,
}

impl Meta {
    pub fn get_result_file_name(&self) -> String {
        format!(
            "sodigy-test-{}{}-{}.json",
            self.commit.commit_hash.get(0..9).unwrap(),
            if self.is_repo_clean { "" } else { "-dirty" },
            format!("{:?}", self.os).to_ascii_lowercase(),
        )
    }
}

#[derive(Clone, Debug, Deserialize, Serialize)]
pub enum Os {
    Linux,
    Mac,
    Windows,
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
        started_at,
    }
}

fn get_os() -> Os {
    if cfg!(target_os = "linux") {
        Os::Linux
    }

    else if cfg!(target_os = "macos") {
        Os::Mac
    }

    else if cfg!(target_os = "windows") {
        Os::Windows
    }

    else {
        Os::Etc
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
