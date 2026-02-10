use regex::Regex;
use serde::{Deserialize, Serialize};
use std::process::Command;

#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct CommitInfo {
    pub title: String,
    pub body: Option<String>,
    pub author: String,
    pub author_email: String,
    pub timestamp: i64,
    pub timezone: String,
    pub tree_hash: String,
    pub commit_hash: String,

    // TODO: multiple parents
    pub parent_hash: Option<String>,
}

pub fn check_clean_repo() -> bool {
    let output = Command::new("git")
        .arg("status")
        .arg("--porcelain")
        .output()
        .unwrap();

    assert!(output.status.success(), "failed to run `git status --porcelain`! Perhaps git is not installed?");

    for line in output.stdout.split(|b| *b == b'\n') {
        if line.len() < 2 {
            continue;
        }

        let prefix = &line[..2];

        // no modification, no add, no deletion
        if prefix.contains(&b'M') || prefix.contains(&b'A') || prefix.contains(&b'D') {
            return false;
        }
    }

    true
}

pub fn get_curr_commit() -> String {
    let output = Command::new("git")
        .arg("rev-list")
        .arg("HEAD")
        .arg("-n")
        .arg("1")
        .output()
        .unwrap();

    assert!(output.status.success());
    String::from_utf8_lossy(&output.stdout).trim().get(0..9).unwrap().to_string()
}

pub fn get_commit_info(commit_hash: &str) -> CommitInfo {
    let output = Command::new("git")
        .arg("cat-file")
        .arg("commit")
        .arg(commit_hash)
        .output()
        .unwrap();

    // TODO: lazy-static?
    let tree_re = Regex::new(r"^tree\s([0-9a-f]+)").unwrap();
    let parent_re = Regex::new(r"^parent\s([0-9a-f]+)").unwrap();
    let committer_re = Regex::new(r"^committer\s([a-zA-Z0-9_@.-]+)\s<([a-zA-Z0-9_@.-]+)>\s(\d+)\s([+-]?\d+)").unwrap();

    assert!(output.status.success());
    let stdout = String::from_utf8_lossy(&output.stdout).to_string();
    let mut reading_commit_message = false;
    let mut commit_message_lines = vec![];
    let mut author = None;
    let mut author_email = None;
    let mut timestamp = None;
    let mut timezone = None;
    let mut tree_hash = None;
    let mut parent_hash = None;

    for line in stdout.lines() {
        if reading_commit_message {
            commit_message_lines.push(line);
        }

        else if let Some(tree) = tree_re.captures(line) {
            tree_hash = Some(tree.get(1).unwrap().as_str().get(0..9).unwrap().to_string());
        }

        else if let Some(parent) = parent_re.captures(line) {
            parent_hash = Some(Some(parent.get(1).unwrap().as_str().get(0..9).unwrap().to_string()));
        }

        else if let Some(committer) = committer_re.captures(line) {
            author = Some(committer.get(1).unwrap().as_str().to_string());
            author_email = Some(committer.get(2).unwrap().as_str().to_string());
            timestamp = Some(committer.get(3).unwrap().as_str().parse::<i64>().unwrap());
            timezone = Some(committer.get(4).unwrap().as_str().to_string());
            reading_commit_message = true;
        }

        else {
            // ignore other data
        }
    }

    let joined_commit_message = commit_message_lines.join("\n");
    commit_message_lines = joined_commit_message.trim().lines().collect();

    let (title, body) = if commit_message_lines.len() > 2 {
        (commit_message_lines[0].to_string(), Some(commit_message_lines[1..].join("\n").trim().to_string()))
    } else {
        (commit_message_lines[0].to_string(), None)
    };

    CommitInfo {
        title,
        body,
        author: author.unwrap(),
        author_email: author_email.unwrap(),
        timestamp: timestamp.unwrap(),
        timezone: timezone.unwrap(),
        tree_hash: tree_hash.unwrap(),
        commit_hash: commit_hash.to_string(),
        parent_hash: parent_hash.unwrap(),
    }
}
