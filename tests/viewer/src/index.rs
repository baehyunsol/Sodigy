use sodigy_compiler_test::{find_root, git, hash_dir};
use sodigy_fs_api::{
    WriteMode,
    create_dir,
    exists,
    join,
    join4,
    remove_dir_all,
    write_bytes,
};
use std::collections::HashSet;

pub fn load_test_files() -> Result<(), String> {
    let root = find_root().map_err(|e| format!("{e:?}"))?;
    let test_files_at = join4(&root, "tests", "log", "test_files").map_err(|e| format!("{e:?}"))?;

    if exists(&test_files_at) {
        remove_dir_all(&test_files_at).map_err(|e| format!("{e:?}"))?;
    }

    create_dir(&test_files_at).map_err(|e| format!("{e:?}"))?;
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
        remove_dir_all("tmp/").map_err(|e| format!("{e:?}"))?;
    }

    create_dir("tmp/").map_err(|e| format!("{e:?}"))?;

    for cnr_tree in cnr_trees.iter() {
        let entries = git::get_tree_info(cnr_tree);

        for entry in entries.iter() {
            if checked_objects.contains(&entry.hash) {
                continue;
            }

            match entry.object_type {
                git::ObjectType::Blob if entry.name.ends_with(".sdg") => {
                    let blob = git::read_blob(&entry.hash);
                    remove_dir_all("tmp/").map_err(|e| format!("{e:?}"))?;
                    create_dir("tmp/").map_err(|e| format!("{e:?}"))?;
                    write_bytes(
                        "tmp/lib.sdg",
                        &blob,
                        WriteMode::AlwaysCreate,
                    ).map_err(|e| format!("{e:?}"))?;
                    let hash = hash_dir("tmp/");
                    let hash = format!("{hash:024x}");
                    let parent_dir = join(&test_files_at, hash.get(0..2).unwrap()).map_err(|e| format!("{e:?}"))?;

                    if !exists(&parent_dir) {
                        create_dir(&parent_dir).map_err(|e| format!("{e:?}"))?;
                    }

                    write_bytes(
                        &join(&parent_dir, hash.get(2..).unwrap()).map_err(|e| format!("{e:?}"))?,
                        &blob,
                        WriteMode::CreateOrTruncate,
                    ).map_err(|e| format!("{e:?}"))?;
                    checked_objects.insert(entry.hash.to_string());
                },
                git::ObjectType::Tree => { /* TODO */ },
                _ => {},
            }
        }
    }

    if exists("tmp/") {
        remove_dir_all("tmp/").map_err(|e| format!("{e:?}"))?;
    }

    Ok(())
}
