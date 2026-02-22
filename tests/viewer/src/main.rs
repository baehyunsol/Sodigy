use regex::Regex;
use serde::{Deserialize, Serialize};
use shev::{
    Color,
    Entries,
    Entry,
    EntryFlag,
    EntryState,
    Filter,
    TextBox,
    Transition,
};
use sodigy_compiler_test::{CompileAndRun, TestHarness, find_root, git};
use sodigy_fs_api::{basename, exists, join, join3, join4, read_dir, read_string};
use std::collections::HashSet;
use std::collections::hash_map::{Entry as HashMapEntry, HashMap};

fn help_message(bin: &str) -> String {
    format!(r#"
{bin} load-test-files
    Loads the test files and quit

{bin}
    Launch the viewer
"#)
}

fn main() {
    let args = std::env::args().collect::<Vec<_>>();

    match args.get(1).map(|arg| arg.as_str()) {
        Some("load-test-files") => {
            load_test_files().unwrap();
            return;
        },
        Some(_) => {
            println!("{}", help_message(&args[0]));
            return;
        },
        None => {},
    }

    let root = find_root().unwrap();
    let test_results_at = join3(&root, "tests", "log").unwrap();
    let (test_results, total_count) = collect_test_result_names(&test_results_at);

    // recent_test_results[0] is the most recent one, and the results are sorted by commit order
    // it collects the most recent 100 results
    let mut recent_test_results = vec![];
    let mut curr_commit = git::get_curr_commit();

    while recent_test_results.len() < 100 {
        let curr_commit_info = git::get_commit_info(&curr_commit);

        if let Some(results) = test_results.get(&curr_commit) {
            recent_test_results.extend(results);

            if recent_test_results.len() == total_count {
                break;
            }
        }

        match curr_commit_info.parent_hash {
            Some(parent) => {
                curr_commit = parent;
            },
            None => break,
        }
    }

    let mut harnesses = vec![];
    let mut entries_map = HashMap::new();

    for recent_test_result in recent_test_results.into_iter() {
        let path = join(&test_results_at, &recent_test_result).unwrap();
        let s = read_string(&path).unwrap();
        let j: TestHarness = serde_json::from_str(&s).unwrap();
        let summ = summary(&j);
        harnesses.push(Entry {
            name: recent_test_result.to_string(),
            content: Some(serde_json::to_string(&summ).unwrap()),
            search_corpus: None,
            categories: vec![],
            transition1: Some(Transition {
                id: recent_test_result.to_string(),
                description: Some(String::from("See details")),
            }),
            transition2: None,
            flag: EntryFlag::None,
        });

        let mut cnr_results = vec![];

        for cnr in j.compile_and_run.as_ref().unwrap_or(&vec![]).iter() {
            cnr_results.push(Entry {
                name: cnr.name.to_string(),
                content: Some(serde_json::to_string(cnr).unwrap()),
                search_corpus: None,
                categories: vec![],
                transition1: None,
                transition2: None,
                flag: if cnr.error.is_some() { EntryFlag::Red } else { EntryFlag::Green },
            });
        }

        entries_map.insert(
            recent_test_result.to_string(),
            Entries {
                id: recent_test_result.to_string(),
                title: Some(recent_test_result.to_string()),
                entries: cnr_results,
                entry_state_count: 2,
                transition: Some(Transition {
                    id: String::from("index"),
                    description: Some(String::from("Back to harnesses")),
                }),
                filters: vec![
                    Filter {
                        name: String::from("pass-only"),
                        cond: |entry| entry.flag == EntryFlag::Green
                    },
                    Filter {
                        name: String::from("fail-only"),
                        cond: |entry| entry.flag == EntryFlag::Red
                    },
                ],
                render_canvas: |entry, entry_state| {
                    let cnr: CompileAndRun = serde_json::from_str(entry.content.as_ref().unwrap()).unwrap();

                    match entry_state {
                        EntryState(0) => {
                            let s = format!(
                                "# stdout\n\n```\n{}\n```\n\n# stderr\n\n```\n{}\n```{}",
                                cnr.stdout_colored,
                                cnr.stderr_colored,
                                if let Some(error) = &cnr.error { format!("\n\n# test error\n\n```\n{error}\n```") } else { String::new() },
                            );
                            let (s, colors) = apply_ansi_term_color(&s);
                            Ok(TextBox::new(
                                &s,
                                16.0,
                                Color { r: 1.0, g: 1.0, b: 1.0, a: 1.0 },
                                [20.0, 20.0, 1080.0, 2000.0],
                            ).with_color_map(colors).render())
                        },
                        EntryState(1) => {
                            let root = find_root().map_err(|e| format!("{e:?}"))?;
                            let test_files_at = join4(&root, "tests", "log", "test_files").map_err(|e| format!("{e:?}"))?;
                            let hash = &cnr.hash;
                            let test_file_at = join3(
                                &test_files_at,
                                hash.get(0..2).ok_or(format!("Corrupted hash: {hash:?}"))?,
                                hash.get(2..).ok_or(format!("Corrupted hash: {hash:?}"))?,
                            ).map_err(|e| format!("{e:?}"))?;

                            if exists(&test_file_at) {
                                Ok(TextBox::new(
                                    &read_string(&test_file_at).map_err(|e| format!("{e:?}"))?,
                                    16.0,
                                    Color { r: 1.0, g: 1.0, b: 1.0, a: 1.0 },
                                    [20.0, 20.0, 1080.0, 2000.0],
                                ).render())
                            }

                            else {
                                Err(format!("File not found: {test_file_at}\nTry running `load-test-files` command!"))
                            }
                        },
                        _ => unreachable!(),
                    }
                },
                ..Entries::default()
            },
        );
    }

    entries_map.insert(
        String::from("index"),
        Entries {
            id: String::from("index"),
            title: Some(String::from("Sodigy-compiler-test")),
            entries: harnesses,
            render_canvas: |entry, _| {
                let entry: TestHarnessSummary = serde_json::from_str(entry.content.as_ref().unwrap()).unwrap();
                Ok(TextBox::new(
                    &format!(
                        "crates: {}/{}\ncompile-and-run: {}/{}\ntested at: {}",
                        entry.crates_pass,
                        entry.crates_pass + entry.crates_fail,
                        entry.cnr_pass,
                        entry.cnr_pass + entry.cnr_fail,
                        entry.started_at,
                    ),
                    16.0,
                    Color { r: 1.0, g: 1.0, b: 1.0, a: 1.0 },
                    [30.0, 30.0, 840.0, 540.0],
                ).render())
            },
            ..Entries::default()
        },
    );
    shev::run(
        shev::Config::default(),
        entries_map,
        String::from("index"),
    );
}

// commit hash to file names map
// there can be multiple files per commit hash because one can run the tests in different OSes.
// it doesn't collect dirty ones
fn collect_test_result_names(dir: &str) -> (HashMap<String, Vec<String>>, usize) {
    let test_result_re = Regex::new(r"sodigy\-test\-([0-9a-f]{9})\-[a-z]+\.json").unwrap();
    let mut result: HashMap<String, Vec<String>> = HashMap::new();
    let mut total_count = 0;

    for file in read_dir(dir, true).unwrap() {
        let name = basename(&file).unwrap();

        if let Some(c) = test_result_re.captures(&name) {
            let hash = c.get(1).unwrap().as_str().to_string();
            total_count += 1;

            match result.entry(hash) {
                HashMapEntry::Occupied(mut e) => {
                    e.get_mut().push(name);
                },
                HashMapEntry::Vacant(e) => {
                    e.insert(vec![name]);
                },
            }
        }
    }

    (result, total_count)
}

fn summary(test_harness: &TestHarness) -> TestHarnessSummary {
    TestHarnessSummary {
        started_at: test_harness.meta.started_at.to_string(),
        crates_pass: test_harness.crates.as_ref().map(|crates| crates.iter().filter(
            |cr| !cr.has_error()
        ).count()).unwrap_or(0),
        crates_fail: test_harness.crates.as_ref().map(|crates| crates.iter().filter(
            |cr| cr.has_error()
        ).count()).unwrap_or(0),
        cnr_pass: test_harness.compile_and_run.as_ref().map(|cnrs| cnrs.iter().filter(
            |cnr| cnr.error.is_none()
        ).count()).unwrap_or(0),
        cnr_fail: test_harness.compile_and_run.as_ref().map(|cnrs| cnrs.iter().filter(
            |cnr| cnr.error.is_some()
        ).count()).unwrap_or(0),
    }
}

#[derive(Deserialize, Serialize)]
struct TestHarnessSummary {
    started_at: String,
    crates_pass: usize,
    crates_fail: usize,
    cnr_pass: usize,
    cnr_fail: usize,
}

#[derive(Clone, Copy)]
enum TermColorParseState {
    Text,
    Control,
}

fn apply_ansi_term_color(s: &str) -> (String, Vec<Color>) {
    let mut chars: Vec<char> = vec![];
    let mut colors = vec![];
    let mut curr_color = Color { r: 1.0, g: 1.0, b: 1.0, a: 1.0 };
    let mut state = TermColorParseState::Text;
    let mut digits_buffer = vec![];

    for ch in s.chars() {
        match state {
            TermColorParseState::Text => match ch {
                '\u{1b}' => {
                    digits_buffer = vec![];
                    state = TermColorParseState::Control;
                },
                _ => {
                    chars.push(ch);
                    colors.push(curr_color);
                },
            },
            TermColorParseState::Control => match ch {
                '0'..='9' => {
                    digits_buffer.push(ch);
                },
                'm' => {
                    match digits_buffer.iter().collect::<String>().parse::<u32>() {
                        Ok(0) => {
                            curr_color = Color { r: 1.0, g: 1.0, b: 1.0, a: 1.0 };
                        },
                        Ok(31) => {
                            curr_color = Color { r: 0.9, g: 0.3, b: 0.3, a: 1.0 };
                        },
                        Ok(32) => {
                            curr_color = Color { r: 0.3, g: 0.9, b: 0.3, a: 1.0 };
                        },
                        Ok(33) => {
                            curr_color = Color { r: 0.9, g: 0.9, b: 0.3, a: 1.0 };
                        },
                        Ok(34) => {
                            curr_color = Color { r: 0.3, g: 0.3, b: 0.9, a: 1.0 };
                        },
                        _ => {},
                    };

                    state = TermColorParseState::Text;
                },
                _ => {},
            },
        }
    }

    (chars.iter().collect(), colors)
}

fn load_test_files() -> Result<(), String> {
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

    // 1. iterate all the tree hashes,
    // 2. run hash_dir (which is defined in runner/src/compile_and_run.rs) for each test files,
    // 3. and save them in `tests/log/test_files/`
    todo!()
}
