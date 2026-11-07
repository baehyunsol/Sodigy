use regex::Regex;
use serde::{Deserialize, Serialize};
use std::process::Command;
use std::sync::LazyLock;

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

static TREE_RE: LazyLock<Regex> = LazyLock::new(|| Regex::new(r"^tree\s([0-9a-f]+)").unwrap());
static PARENT_RE: LazyLock<Regex> = LazyLock::new(|| Regex::new(r"^parent\s([0-9a-f]+)").unwrap());
static COMMITTER_RE: LazyLock<Regex> = LazyLock::new(|| Regex::new(r"^committer\s([a-zA-Z0-9_@.-]+)\s<([a-zA-Z0-9_@.-]+)>\s(\d+)\s([+-]?\d+)").unwrap());

pub fn get_commit_info(commit_hash: &str) -> CommitInfo {
    let output = Command::new("git")
        .arg("cat-file")
        .arg("commit")
        .arg(commit_hash)
        .output()
        .unwrap();

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

        else if let Some(tree) = TREE_RE.captures(line) {
            tree_hash = Some(tree.get(1).unwrap().as_str().get(0..9).unwrap().to_string());
        }

        else if let Some(parent) = PARENT_RE.captures(line) {
            parent_hash = Some(parent.get(1).unwrap().as_str().get(0..9).unwrap().to_string());
        }

        else if let Some(committer) = COMMITTER_RE.captures(line) {
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
        parent_hash: parent_hash,
    }
}

static TREE_INFO_RE: LazyLock<Regex> = LazyLock::new(|| Regex::new(r"(\d{6})\s([a-z]+)\s([0-9a-f]+)\s(.+)").unwrap());

#[derive(Clone, Debug)]
pub struct TreeEntry {
    pub mode: u32,  // TODO: parse this
    pub object_type: ObjectType,
    pub hash: String,
    pub name: String,
}

pub fn get_tree_info(tree_hash: &str) -> Vec<TreeEntry> {
    let output = Command::new("git")
        .arg("cat-file")
        .arg("-p")
        .arg(tree_hash)
        .output()
        .unwrap();

    assert!(output.status.success());
    let stdout = String::from_utf8_lossy(&output.stdout).to_string();
    let mut result = vec![];

    for line in stdout.lines() {
        let c = TREE_INFO_RE.captures(line).unwrap();
        let mode = u32::from_str_radix(c.get(1).unwrap().as_str(), 8).unwrap();
        let object_type = match c.get(2).unwrap().as_str() {
            "blob" => ObjectType::Blob,
            "tree" => ObjectType::Tree,
            _ => unreachable!(),
        };
        let hash = c.get(3).unwrap().as_str().get(0..9).unwrap().to_string();
        let name = c.get(4).unwrap().as_str().to_string();

        result.push(TreeEntry { mode, object_type, hash, name });
    }

    result
}

// TODO: there must be more types
#[derive(Clone, Debug)]
pub enum ObjectType {
    Blob,
    Tree,
}

pub fn read_blob(blob_hash: &str) -> Vec<u8> {
    let output = Command::new("git")
        .arg("cat-file")
        .arg("blob")
        .arg(blob_hash)
        .output()
        .unwrap();

    assert!(output.status.success());
    output.stdout
}

#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct CommitDiff {
    pub commits: (String, String),
    pub add: usize,
    pub remove: usize,
    pub diffs: Vec<((String, String), Vec<Hunk>)>,
}

#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct Hunk {
    pub add: usize,
    pub remove: usize,
    pub line_diffs: Vec<LineDiff>,
}

#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct LineDiff {
    pub kind: DiffKind,
    pub line: String,
}

#[derive(Clone, Debug, Deserialize, Serialize)]
pub enum DiffKind {
    Add,
    Remove,
    Context,
}

static LINE_HEADER_RE: LazyLock<Regex> = LazyLock::new(|| Regex::new(r#"diff --git "?a/(.+)"? "?b/(.+)"?"#).unwrap());

pub fn get_diff(commit_a: &str, commit_b: &str) -> CommitDiff {
    let output = Command::new("git")
        .arg("diff")
        .arg(commit_a)
        .arg(commit_b)
        .arg("--diff-algorithm=patience")
        .arg("-U5")
        .output()
        .unwrap();

    assert!(output.status.success());
    let output: &str = &String::from_utf8_lossy(&output.stdout);

    let mut add = 0;
    let mut remove = 0;
    let mut diffs = vec![];

    let mut curr_file_a = String::new();
    let mut curr_file_b = String::new();
    let mut curr_hunks = vec![];
    let mut curr_hunk = vec![];
    let mut curr_hunk_add = 0;
    let mut curr_hunk_remove = 0;
    let mut ignore_until_triple_plus = false;

    for line in output.lines() {
        if ignore_until_triple_plus {
            if line.starts_with("+++ ") {
                ignore_until_triple_plus = false;
            }
        }

        else if let Some(cap) = LINE_HEADER_RE.captures(line) {
            if !curr_hunk.is_empty() {
                curr_hunks.push(Hunk {
                    add: curr_hunk_add,
                    remove: curr_hunk_remove,
                    line_diffs: std::mem::take(&mut curr_hunk),
                });
                add += curr_hunk_add;
                remove += curr_hunk_remove;
            }

            if !curr_hunks.is_empty() {
                diffs.push(((curr_file_a.clone(), curr_file_b.clone()), std::mem::take(&mut curr_hunks)));
            }

            curr_file_a = cap.get(1).unwrap().as_str().to_string();
            curr_file_b = cap.get(2).unwrap().as_str().to_string();
            ignore_until_triple_plus = true;
        }

        else if line.starts_with("@@") {
            if !curr_hunk.is_empty() {
                curr_hunks.push(Hunk {
                    add: curr_hunk_add,
                    remove: curr_hunk_remove,
                    line_diffs: std::mem::take(&mut curr_hunk),
                });
                add += curr_hunk_add;
                remove += curr_hunk_remove;
                curr_hunk_add = 0;
                curr_hunk_remove = 0;
            }
        }

        else if line.starts_with("+") {
            curr_hunk.push(LineDiff {
                kind: DiffKind::Add,
                line: line.get(1..).unwrap().to_string(),
            });
            curr_hunk_add += 1;
        }

        else if line.starts_with("-") {
            curr_hunk.push(LineDiff {
                kind: DiffKind::Remove,
                line: line.get(1..).unwrap().to_string(),
            });
            curr_hunk_remove += 1;
        }

        else if line.starts_with(" ") {
            curr_hunk.push(LineDiff {
                kind: DiffKind::Context,
                line: line.get(1..).unwrap().to_string(),
            });
        }

        else if line.starts_with("\\ No newline") {}

        else {
            panic!("{line:?}")
        }
    }

    if !curr_hunk.is_empty() {
        curr_hunks.push(Hunk {
            add: curr_hunk_add,
            remove: curr_hunk_remove,
            line_diffs: std::mem::take(&mut curr_hunk),
        });
        add += curr_hunk_add;
        remove += curr_hunk_remove;
    }

    if !curr_hunks.is_empty() {
        diffs.push(((curr_file_a.clone(), curr_file_b.clone()), std::mem::take(&mut curr_hunks)));
    }

    CommitDiff {
        commits: (commit_a.to_string(), commit_b.to_string()),
        add,
        remove,
        diffs,
    }
}

