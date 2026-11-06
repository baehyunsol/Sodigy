use lazy_static::lazy_static;
use regex::Regex;
use sodigy_fs_api::{FileError, basename, join, read_dir, read_string};
use sodigy_compiler_test::{
    CompileAndRun,
    CrateTest,
    CrateTestUnit,
    FuzzResult,
    TestHarness,
    meta::git,
};
use std::collections::hash_map::{Entry, HashMap};
use viewer::{Color, Graphic, Id, Node, render_all};

mod s_utils;

use s_utils::render_ms;

fn main() {
    let commits = load_commits();
    let results = load_test_results().unwrap();
    let mut id = Id::init();
    let mut nodes = vec![];
    let mut top_siblings = Vec::with_capacity(commits.len());

    for commit in commits.iter() {
        let title = format!("{}: {}", commit.commit_hash, commit.title);
        let commit_node_id = render_commit(commit, &results, &mut id, &mut nodes);
        top_siblings.push((title, commit_node_id));
    }

    id.increment();
    nodes.push(Node {
        id,
        title: String::from("Sodigy CI"),
        flag: None,
        metadata: None,
        children: Some(top_siblings),
    });
    render_all(&nodes, id, "../log/rendered").unwrap();
}

fn load_commits() -> Vec<git::CommitInfo> {
    let mut curr_commit = git::get_curr_commit();
    let mut result = vec![];

    for _ in 0..300 {
        let commit_info = git::get_commit_info(&curr_commit);
        result.push(commit_info.clone());

        match &commit_info.parent_hash {
            Some(parent) => { curr_commit = parent.clone(); },
            None => break,
        }
    }

    result
}

fn render_commit(
    commit: &git::CommitInfo,
    test_results: &HashMap<String, Vec<TestHarness>>,
    id: &mut Id,
    nodes: &mut Vec<Node>,
) -> Id {
    let (flag, children) = if let Some(results) = test_results.get(&commit.commit_hash) {
        let mut children = Vec::with_capacity(results.len());
        let mut has_error = false;

        for result in results.iter() {
            let result_node_id = render_test_result(result, id, nodes);
            let title = format!("{:?}-{:?}", result.meta.os, result.meta.isa).to_ascii_lowercase();
            has_error |= result.has_error();

            children.push((title, result_node_id));
        }

        let flag = if has_error { Some(Color::Red) } else { Some(Color::Green) };
        (flag, Some(children))
    } else {
        (None, None)
    };
    let mut metadata = vec![
        Graphic::TextBox(vec![(None, format!("{}\n\n{}", commit.title, commit.body.as_ref().unwrap_or(&String::new())))]),
    ];

    if let Some(parent_hash) = &commit.parent_hash {
        let diff = git::get_diff(&commit.commit_hash, parent_hash);
        let diff_node_id = render_commit_diff(&diff, format!("{parent_hash} vs {}", commit.commit_hash), id, nodes);
        metadata.push(Graphic::Link { text: String::from("updated files"), id: diff_node_id });
    }

    // TODO: render file/dir

    id.increment();
    nodes.push(Node {
        id: *id,
        title: commit.commit_hash.to_string(),
        flag,
        metadata: Some(metadata),
        children,
    });
    *id
}

fn render_commit_diff(
    diff: &str,
    title: String,
    id: &mut Id,
    nodes: &mut Vec<Node>,
) -> Id {
    let mut lines = vec![];

    for line in diff.lines() {
        if line.starts_with("+") {
            lines.push((Some(Color::Green), format!("{line}\n")));
        } else if line.starts_with("-") {
            lines.push((Some(Color::Red), format!("{line}\n")));
        } else if line.starts_with("@") {
            lines.push((Some(Color::Yellow), format!("{line}\n")));
        } else {
            lines.push((None, format!("{line}\n")));
        }
    }

    id.increment();
    nodes.push(Node {
        id: *id,
        title,
        flag: None,
        metadata: Some(vec![Graphic::TextBox(lines)]),
        children: None,
    });
    *id
}

lazy_static! {
    static ref TEST_RESULT_RE: Regex = Regex::new(r"sodigy-test-([0-9a-f]+)-([^-]+)-([^-]+)\.json").unwrap();
}

fn load_test_results() -> Result<HashMap<String /* commit_hash */, Vec<TestHarness>>, FileError> {
    let results_at = join("..", "log")?;
    let mut results: HashMap<String, Vec<TestHarness>> = HashMap::new();

    for file in read_dir(&results_at, true)? {
        if let Some(cap) = TEST_RESULT_RE.captures(&basename(&file)?) {
            let result: TestHarness = serde_json::from_str(&read_string(&file)?).unwrap();

            match results.entry(result.meta.commit.commit_hash.to_string()) {
                Entry::Occupied(mut e) => {
                    e.get_mut().push(result);
                },
                Entry::Vacant(e) => {
                    e.insert(vec![result]);
                },
            }
        }
    }

    Ok(results)
}

fn render_test_result(result: &TestHarness, id: &mut Id, nodes: &mut Vec<Node>) -> Id {
    let flag = if result.has_error() { Some(Color::Red) } else { Some(Color::Green) };
    let mut children = vec![];

    if let Some(crates) = &result.crates {
        let crates_node_id = render_crates_result(crates, id, nodes);
        children.push((String::from("crates"), crates_node_id));
    }

    if let Some(cnr) = &result.compile_and_run {
        let cnr_node_id = render_cnrs_result(cnr, id, nodes);
        children.push((String::from("cnrs"), cnr_node_id));
    }

    if let Some(fuzz) = &result.fuzz {
        let fuzz_node_id = render_fuzzs_result(fuzz, id, nodes);
        children.push((String::from("fuzz"), fuzz_node_id));
    }

    id.increment();
    nodes.push(Node {
        id: *id,
        title: result.meta.get_result_file_name(),
        flag,
        metadata: None,
        children: Some(children),
    });
    *id
}

fn render_crates_result(crates: &[CrateTest], id: &mut Id, nodes: &mut Vec<Node>) -> Id {
    let flag = if crates.iter().any(|c| c.has_error()) { Some(Color::Red) } else { Some(Color::Green) };
    let mut children = vec![];

    for cr in crates.iter() {
        let cr_node_id = render_crate_result(cr, id, nodes);
        children.push((cr.name.to_string(), cr_node_id));
    }

    id.increment();
    nodes.push(Node {
        id: *id,
        title: String::from("crate tests"),
        flag,
        metadata: None,
        children: Some(children),
    });
    *id
}

fn render_crate_result(cr: &CrateTest, id: &mut Id, nodes: &mut Vec<Node>) -> Id {
    let flag = if cr.has_error() { Some(Color::Red) } else { Some(Color::Green) };
    let mut children = vec![];

    for unit in cr.tests.iter() {
        let unit_node_id = render_crate_unit_result(unit, id, nodes);
        children.push((unit.command.to_string(), unit_node_id));
    }

    id.increment();
    nodes.push(Node {
        id: *id,
        title: cr.name.to_string(),
        flag,
        metadata: None,
        children: Some(children),
    });
    *id
}

fn render_crate_unit_result(unit: &CrateTestUnit, id: &mut Id, nodes: &mut Vec<Node>) -> Id {
    fn color_errors_and_warnings(stderr: &str) -> Vec<(Option<Color>, String)> {
        let mut result = vec![];

        for line in stderr.lines() {
            if line.starts_with("error: ") {
                result.push((Some(Color::Red), String::from("error: ")));
                result.push((None, line.get(7..).unwrap().to_string()));
            } else if line.starts_with("error[E") {
                result.push((Some(Color::Red), String::from("error")));
                result.push((None, line.get(5..).unwrap().to_string()));
            } else if line.starts_with("warning: ") {
                result.push((Some(Color::Yellow), String::from("warning: ")));
                result.push((None, line.get(9..).unwrap().to_string()));
            } else {
                result.push((None, line.to_string()));
            }

            result.push((None, String::from("\n")));
        }

        result
    }

    let flag = if unit.has_error { Some(Color::Red) } else { Some(Color::Green) };
    let metadata = vec![
        Graphic::Title(String::from("command")),
        Graphic::TextBox(vec![(None, format!("cargo {}", unit.command))]),
        Graphic::Title(String::from("elapsed time")),
        Graphic::Text(render_ms(unit.elapsed_ms)),
        Graphic::Title(String::from("stdout")),
        Graphic::TextBox(vec![(None, unit.stdout.to_string())]),
        Graphic::Title(String::from("stderr")),
        Graphic::TextBox(color_errors_and_warnings(&unit.stderr)),

    ];

    id.increment();
    nodes.push(Node {
        id: *id,
        title: unit.command.to_string(),
        flag,
        metadata: Some(metadata),
        children: None,
    });
    *id
}

fn render_cnrs_result(cnrs: &[CompileAndRun], id: &mut Id, nodes: &mut Vec<Node>) -> Id {
    let (mut pass, mut fail) = (0, 0);

    for cnr in cnrs.iter() {
        if cnr.has_error() { fail += 1; } else { pass += 1; }
    }

    let flag = if fail > 0 { Some(Color::Red) } else { Some(Color::Green) };
    let mut children = vec![];

    for cnr in cnrs.iter() {
        let cnr_node_id = render_cnr_result(cnr, id, nodes);
        children.push((cnr.name.to_string(), cnr_node_id));
    }

    id.increment();
    nodes.push(Node {
        id: *id,
        title: String::from("Compile & Run tests"),
        flag,
        metadata: Some(vec![Graphic::Text(format!("pass: {pass}, fail: {fail}"))]),
        children: Some(children),
    });
    *id
}

fn render_cnr_result(cnr: &CompileAndRun, id: &mut Id, nodes: &mut Vec<Node>) -> Id {
    // copy-pasted from sodigy-driver/log.rs
    enum TermColorParseState {
        Text,
        Control,
    }

    fn color_ansi_term(s: &str) -> Vec<(Option<Color>, String)> {
        let mut state = TermColorParseState::Text;
        let mut content_buffer: Vec<char> = vec![];
        let mut digits_buffer: Vec<char> = vec![];
        let mut result = vec![];
        let mut curr_color = None;

        for ch in s.chars() {
            match state {
                TermColorParseState::Text => match ch {
                    '\u{1b}' => {
                        digits_buffer = vec![];
                        result.push((curr_color, content_buffer.drain(..).collect()));
                        state = TermColorParseState::Control;
                    },
                    _ => {
                        content_buffer.push(ch);
                    },
                },
                TermColorParseState::Control => match ch {
                    '0'..='9' => {
                        digits_buffer.push(ch);
                    },
                    'm' => {
                        match digits_buffer.iter().collect::<String>().parse::<u32>() {
                            Ok(0) => {
                                curr_color = None;
                            },
                            Ok(31) => {
                                curr_color = Some(Color::Red);
                            },
                            Ok(32) => {
                                curr_color = Some(Color::Green);
                            },
                            Ok(33) => {
                                curr_color = Some(Color::Yellow);
                            },
                            Ok(34) => {
                                curr_color = Some(Color::Blue);
                            },
                            _ => unreachable!(),
                        };

                        state = TermColorParseState::Text;
                    },
                    _ => {},
                },
            }
        }

        if !content_buffer.is_empty() {
            result.push((curr_color, content_buffer.drain(..).collect()));
        }

        result
    }

    let flag = if cnr.has_error() { Some(Color::Red) } else { Some(Color::Green) };
    let mut metadata = vec![];

    if let Some(error) = &cnr.error {
        metadata.push(Graphic::Title(String::from("Error")));
        metadata.push(Graphic::TextBox(vec![(None, error.to_string())]));
    }

    metadata.push(Graphic::Title(String::from("Build elapsed time")));
    metadata.push(Graphic::Text(render_ms(cnr.build_elapsed_ms)));

    if let Some(ms) = cnr.run_elapsed_ms {
        metadata.push(Graphic::Title(String::from("Run elapsed time")));
        metadata.push(Graphic::Text(render_ms(ms)));
    }

    if let Some(ms) = cnr.test_elapsed_ms {
        metadata.push(Graphic::Title(String::from("Test elapsed time")));
        metadata.push(Graphic::Text(render_ms(ms)));
    }

    for (title, content) in [
        ("Build stdout", &cnr.build_stdout_colored),
        ("Build stderr", &cnr.build_stderr_colored),
        ("Run stdout", &cnr.run_stdout_colored),
        ("Run stderr", &cnr.run_stderr_colored),
        ("Test stdout", &cnr.test_stdout_colored),
        ("Test stderr", &cnr.test_stderr_colored),
    ] {
        if let Some(content) = content {
            metadata.push(Graphic::Title(title.to_string()));
            metadata.push(Graphic::TextBox(color_ansi_term(content)));
        }
    }

    id.increment();
    nodes.push(Node {
        id: *id,
        title: cnr.name.to_string(),
        flag,
        metadata: Some(metadata),
        children: None,
    });
    *id
}

fn render_fuzzs_result(fuzzs: &[FuzzResult], id: &mut Id, nodes: &mut Vec<Node>) -> Id {
    let flag = if fuzzs.iter().any(|f| f.has_error()) { Some(Color::Red) } else { Some(Color::Green) };
    let mut children = vec![];

    for fuzz in fuzzs.iter() {
        let fuzz_node_id = render_fuzz_result(fuzz, id, nodes);
        children.push((fuzz.target.name().to_string(), fuzz_node_id));
    }

    id.increment();
    nodes.push(Node {
        id: *id,
        title: String::from("fuzz"),
        flag,
        metadata: None,
        children: Some(children),
    });
    *id
}

fn render_fuzz_result(fuzz: &FuzzResult, id: &mut Id, nodes: &mut Vec<Node>) -> Id {
    let flag = if fuzz.has_error() { Some(Color::Red) } else { Some(Color::Green) };
    let mut metadata = vec![
        Graphic::Title(String::from("elapsed ms")),
        Graphic::Text(render_ms(fuzz.elapsed_ms)),
    ];

    if let Some(artifact) = &fuzz.artifact {
        let raw_bytes = if artifact.len() < 16 {
            format!("{artifact:?}")
        } else {
            let mut lines = Vec::with_capacity(artifact.len() / 8 + 1);

            for chunk in artifact.chunks(8) {
                lines.push(format!("    {},", chunk.iter().map(|b| format!("0x{b:02x}")).collect::<Vec<_>>().join(", ")));
            }

            format!("[\n{}\n]", lines.join("\n"))
        };

        metadata.push(Graphic::Title(String::from("artifact")));
        metadata.push(Graphic::TextBox(vec![(None, String::from_utf8_lossy(artifact).to_string())]));
        metadata.push(Graphic::Title(String::from("artifact (raw)")));
        metadata.push(Graphic::TextBox(vec![(None, raw_bytes)]));
    }

    id.increment();
    nodes.push(Node {
        id: *id,
        title: fuzz.target.name().to_string(),
        flag,
        metadata: Some(metadata),
        children: None,
    });
    *id
}

