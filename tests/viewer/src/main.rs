use regex::Regex;
use sodigy_compiler_test::{TestHarness, find_root, git};
use sodigy_fs_api::{
    basename,
    create_dir_all,
    exists,
    join,
    join3,
    parent,
    read_dir,
    read_string,
    remove_dir_all,
    set_extension,
    write_string,
    WriteMode,
};
use std::collections::hash_map::{Entry as HashMapEntry, HashMap};
use std::collections::hash_set::HashSet;

mod blob;
mod commit;
mod harness;
mod index;
mod utils;

use blob::{load_test_files, render_blob};
use commit::render_commit;
use harness::render_harness;
use index::render_index;
use utils::{circle, escape_html, html_template, render_elapsed_ms};

fn main() {
    let root = find_root().unwrap();
    let test_results_at = join3(&root, "tests", "log").unwrap();
    let rendered_htmls_at = join(&parent(&test_results_at).unwrap(), "html").unwrap();
    let (test_results, total_count) = collect_test_result_names(&test_results_at);

    // `recent_test_results[0]` is the most recent one, and the results are sorted by commit order.
    let mut recent_test_results = vec![];
    let mut harnesses_by_name = HashMap::new();
    let mut commits = vec![];
    let mut blobs_to_read = HashSet::new();
    let mut curr_commit = git::get_curr_commit();

    while recent_test_results.len() < 200 && commits.len() < 500 {
        let curr_commit_info = git::get_commit_info(&curr_commit);
        commits.push(curr_commit_info.clone());

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

    if exists(&rendered_htmls_at) {
        remove_dir_all(&rendered_htmls_at).unwrap();
    }

    create_dir_all(&rendered_htmls_at).unwrap();
    create_dir_all(&join(&rendered_htmls_at, "harnesses").unwrap()).unwrap();
    create_dir_all(&join(&rendered_htmls_at, "commits").unwrap()).unwrap();
    create_dir_all(&join(&rendered_htmls_at, "blobs").unwrap()).unwrap();

    for (i, test_result) in recent_test_results.iter().enumerate() {
        let (prev, next) = match i {
            0 => (
                None,
                recent_test_results.get(i + 1).map(|c| c.to_string()),
            ),
            _ => (
                recent_test_results.get(i - 1).map(|c| c.to_string()),
                recent_test_results.get(i + 1).map(|c| c.to_string()),
            ),
        };

        let path = join(&test_results_at, test_result).unwrap();
        let s = read_string(&path).unwrap();
        let j: TestHarness = serde_json::from_str(&s).unwrap();
        harnesses_by_name.insert(test_result.to_string(), j.clone());
        blobs_to_read.extend(j.get_cnr_blobs());
        let html_name = set_extension(&j.meta.get_result_file_name(), "html").unwrap();

        let html = render_harness(&j, prev, next);
        write_string(
            &join3(
                &rendered_htmls_at,
                "harnesses",
                &html_name,
            ).unwrap(),
            &html,
            WriteMode::AlwaysCreate,
        ).unwrap();
    }

    let blobs = load_test_files().unwrap();

    for blob_hash in blobs_to_read.iter() {
        let Some(html) = render_blob(blob_hash, &blobs) else { continue };

        write_string(
            &join3(
                &rendered_htmls_at,
                "blobs",
                &set_extension(blob_hash, "html").unwrap(),
            ).unwrap(),
            &html,
            WriteMode::AlwaysCreate,
        ).unwrap();
    }

    write_string(
        &join(&rendered_htmls_at, "index.html").unwrap(),
        &render_index(&test_results, &commits),
        WriteMode::AlwaysCreate,
    ).unwrap();

    // It takes the longest time, so we do this at the end.
    for commit in commits.iter() {
        let abbrev_hash = commit.commit_hash.get(0..9).unwrap().to_string();

        write_string(
            &join3(
                &rendered_htmls_at,
                "commits",
                &set_extension(&abbrev_hash, "html").unwrap(),
            ).unwrap(),
            &render_commit(commit),
            WriteMode::AlwaysCreate,
        ).unwrap();
    }
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
