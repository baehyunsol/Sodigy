use crate::collect_test_result_names;
use serde::{Deserialize, Serialize};
use sodigy_compiler_test::{CompileAndRun, TestHarness, find_root, git, hash_dir};
use sodigy_fs_api::{
    FileError,
    WriteMode,
    create_dir,
    exists,
    join,
    join3,
    join4,
    read_string,
    remove_dir_all,
    write_bytes,
};
use std::collections::{HashMap, HashSet};

pub fn calc_cnr_diffs() -> Result<(), FileError> {
    let root = find_root()?;
    let test_results_at = join3(&root, "tests", "log")?;
    let test_diffs_at = join(&test_results_at, "diffs")?;

    if exists(&test_diffs_at) {
        remove_dir_all(&test_diffs_at)?;
    }

    create_dir(&test_diffs_at)?;
    let (test_results, mut count) = collect_test_result_names(&test_results_at);
    let mut ordered_test_results = vec![];
    let mut curr_commit_hash = git::get_curr_commit();

    loop {
        let commit_info = git::get_commit_info(&curr_commit_hash);

        if let Some(test_results) = test_results.get(&curr_commit_hash) {
            ordered_test_results.extend(test_results);
            count -= 1;
        }

        if count == 0 {
            break;
        }

        match commit_info.parent_hash {
            Some(parent) => {
                curr_commit_hash = parent;
            },
            None => {
                break;
            },
        }
    }

    for adjacent_results in ordered_test_results.windows(2) {
        let [next, prev] = adjacent_results else { unreachable!() };
        let diff = calc_cnr_diff(
            &join(&test_results_at, prev)?,
            &join(&test_results_at, next)?,
        )?;

        write_bytes(
            &join(&test_diffs_at, &format!("{prev}-{next}"))?,
            &serde_json::to_vec(&diff).unwrap(),
            WriteMode::CreateOrTruncate,
        )?;
    }

    Ok(())
}

#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct CnrDiff {
    pub prev_hash: String,
    pub next_hash: String,
    pub new_passes: Vec<(Option<CompileAndRun>, CompileAndRun)>,  // Vec<(prev_cnr, next_cnr)>
    pub new_fails: Vec<(Option<CompileAndRun>, CompileAndRun)>,

    // TODO: nice way to compare outputs
    //       naively comparing stderr/stdout is to unstable
    //       How about comparing bytecode outputs?
    pub changes: Vec<(CompileAndRun, CompileAndRun)>,
}

fn calc_cnr_diff(prev: &str, next: &str) -> Result<CnrDiff, FileError> {
    let prev_harness = read_string(prev)?;
    let prev_harness: TestHarness = serde_json::from_str(&prev_harness).unwrap();
    let prev_cnrs: HashMap<String, CompileAndRun> = prev_harness.compile_and_run.as_ref().unwrap_or(&vec![]).iter().map(
        |cnr| (cnr.name.to_string(), cnr.clone())
    ).collect();
    let next_harness = read_string(next)?;
    let next_harness: TestHarness = serde_json::from_str(&next_harness).unwrap();
    let next_cnrs: HashMap<String, CompileAndRun> = next_harness.compile_and_run.as_ref().unwrap_or(&vec![]).iter().map(
        |cnr| (cnr.name.to_string(), cnr.clone())
    ).collect();
    let mut new_passes = vec![];
    let mut new_fails = vec![];
    let mut changes = vec![];

    for (name, next_cnr) in next_cnrs.iter() {
        match prev_cnrs.get(name) {
            Some(prev_cnr) => match (prev_cnr.error.is_some(), next_cnr.error.is_some()) {
                // A failing case passed
                (true, false) => {
                    new_passes.push((Some(prev_cnr.clone()), next_cnr.clone()));
                },

                // A passing case failed
                (false, true) => {
                    new_fails.push((Some(prev_cnr.clone()), next_cnr.clone()));
                },

                // TODO: check if the outputs are the same
                _ => {},
            },
            None => {
                // A test case is added and it failed
                if next_cnr.error.is_some() {
                    new_fails.push((None, next_cnr.clone()));
                }

                // A test case is added and it passed
                else {
                    new_passes.push((None, next_cnr.clone()));
                }
            },
        }
    }

    Ok(CnrDiff {
        prev_hash: prev_harness.meta.commit.commit_hash.clone(),
        next_hash: next_harness.meta.commit.commit_hash.clone(),
        new_passes,
        new_fails,
        changes,
    })
}

pub fn load_test_files() -> Result<(), FileError> {
    let root = find_root()?;
    let test_files_at = join4(&root, "tests", "log", "test_files")?;

    if exists(&test_files_at) {
        remove_dir_all(&test_files_at)?;
    }

    create_dir(&test_files_at)?;
    let curr_commit = git::get_curr_commit();
    let mut curr_commit = git::get_commit_info(&curr_commit);
    let mut cnr_trees = HashSet::new();

    loop {
        let curr_tree = &curr_commit.tree_hash;
        let curr_tree_info = git::get_tree_info(&curr_tree);
        let mut tests_tree_hash = None;

        for entry in curr_tree_info.iter() {
            if entry.name == "tests" {
                tests_tree_hash = Some(entry.hash.clone());
                break;
            }
        }

        match tests_tree_hash {
            Some(hash) => {
                let tests_tree_info = git::get_tree_info(&hash);
                let mut cnr_tree_hash = None;

                for entry in tests_tree_info.iter() {
                    if entry.name == "compile-and-run" {
                        cnr_tree_hash = Some(entry.hash.clone());
                        break;
                    }
                }

                match cnr_tree_hash {
                    Some(hash) => {
                        cnr_trees.insert(hash);

                        match curr_commit.parent_hash {
                            Some(parent) => {
                                curr_commit = git::get_commit_info(&parent);
                            },
                            None => {
                                break;
                            },
                        }
                    },
                    None => {
                        break;
                    },
                }
            },
            None => {
                break;
            },
        }
    }

    // `entry.hash` is git-hash, while `hash_dir` uses its own hash. don't get confused!
    let mut checked_objects: HashSet<String> = HashSet::new();

    if exists("tmp/") {
        remove_dir_all("tmp/")?;
    }

    create_dir("tmp/")?;

    for cnr_tree in cnr_trees.iter() {
        let entries = git::get_tree_info(cnr_tree);

        for entry in entries.iter() {
            if checked_objects.contains(&entry.hash) {
                continue;
            }

            match entry.object_type {
                git::ObjectType::Blob if entry.name.ends_with(".sdg") => {
                    let blob = git::read_blob(&entry.hash);
                    remove_dir_all("tmp/")?;
                    create_dir("tmp/")?;
                    write_bytes(
                        "tmp/lib.sdg",
                        &blob,
                        WriteMode::AlwaysCreate,
                    )?;
                    let hash = hash_dir("tmp/");
                    let hash = format!("{hash:024x}");
                    let parent_dir = join(&test_files_at, hash.get(0..2).unwrap())?;

                    if !exists(&parent_dir) {
                        create_dir(&parent_dir)?;
                    }

                    write_bytes(
                        &join(&parent_dir, hash.get(2..).unwrap())?,
                        &blob,
                        WriteMode::CreateOrTruncate,
                    )?;
                    checked_objects.insert(entry.hash.to_string());
                },
                git::ObjectType::Tree => { /* TODO */ },
                _ => {},
            }
        }
    }

    if exists("tmp/") {
        remove_dir_all("tmp/")?;
    }

    Ok(())
}
