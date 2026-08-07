use crate::{color_udiff, escape_html, html_template};
use sodigy_compiler_test::{CompileAndRun, TestHarness, subprocess};
use sodigy_fs_api::{
    WriteMode,
    remove_file,
    write_bytes,
};
use std::collections::{HashMap, HashSet};

pub fn render_diff(
    prev: &TestHarness,
    next: &TestHarness,
    blobs: &HashMap<String, Vec<u8>>,
) -> String {
    let cnr_diff = match (prev, next) {
        (
            TestHarness { compile_and_run: Some(prev), .. },
            TestHarness { compile_and_run: Some(next), .. },
        ) => {
            let prev_by_name: HashMap<String, &CompileAndRun> = prev.iter().map(
                |cnr| (cnr.name.to_string(), cnr)
            ).collect();
            let next_by_name: HashMap<String, &CompileAndRun> = next.iter().map(
                |cnr| (cnr.name.to_string(), cnr)
            ).collect();
            let mut all_names: Vec<String> = prev.iter().chain(next.iter()).map(
                |cnr| cnr.name.to_string()
            ).collect::<HashSet<_>>().into_iter().collect();
            all_names.sort();

            // Vec<(cnr_name, content_diff, prev_content_link, next_content_link)>
            let mut updated_cnrs: Vec<(String, String, String, String)> = vec![];

            // Vec<(cnr_name, content_link, prev_error)>
            let mut fixed_cnrs: Vec<(String, String, String)> = vec![];

            // Vec<(cnr_name, content_link, next_error)>
            let mut regressed_cnrs: Vec<(String, String, String)> = vec![];

            // Vec<(cnr_name, content_link, prev_error, next_error)>
            let mut different_error_cnrs: Vec<(String, String, String, String)> = vec![];

            // Vec<(cnr_name, content_link)>
            let mut added_cnrs: Vec<(String, String)> = vec![];

            // Vec<(cnr_name, content_link)>
            let mut removed_cnrs: Vec<(String, String)> = vec![];

            for cnr_name in all_names.iter() {
                match (prev_by_name.get(cnr_name), next_by_name.get(cnr_name)) {
                    (Some(prev), Some(next)) => {
                        if prev.hash != next.hash {
                            match (blobs.get(&prev.hash), blobs.get(&next.hash)) {
                                (Some(prev_blob), Some(next_blob)) => {
                                    updated_cnrs.push((cnr_name.to_string(), escape_html(&diff_blob(prev_blob, next_blob)), prev.hash.to_string(), next.hash.to_string()));
                                },
                                // If the test is a multi-file case, it's TODO.
                                _ => {},
                            }
                        }

                        match (&prev.error, &next.error) {
                            (Some(prev_error), Some(next_error)) if prev_error != next_error => {
                                different_error_cnrs.push((cnr_name.to_string(), next.hash.to_string(), escape_html(prev_error), escape_html(next_error)));
                            },
                            (Some(prev_error), None) => {
                                fixed_cnrs.push((cnr_name.to_string(), next.hash.to_string(), escape_html(prev_error)));
                            },
                            (None, Some(next_error)) => {
                                regressed_cnrs.push((cnr_name.to_string(), next.hash.to_string(), escape_html(next_error)));
                            },
                            _ => {},
                        }
                    },
                    (Some(prev), None) => {
                        removed_cnrs.push((cnr_name.to_string(), prev.hash.to_string()));
                    },
                    (None, Some(next)) => {
                        added_cnrs.push((cnr_name.to_string(), next.hash.to_string()));
                    },
                    (None, None) => unreachable!(),
                }
            }

            let updated_count = updated_cnrs.len();
            let updated = updated_cnrs.iter().map(
                |(cnr_name, content_diff, prev_content_link, next_content_link)| {
                    format!(r#"
<h4>{cnr_name}</h4>

<p><a href="../blobs/{prev_content_link}.html">prev code</a></p>
<p><a href="../blobs/{next_content_link}.html">next code</a></p>

<pre class="code-block"><code>
{content_diff}
</code></pre>
"#)
                }
            ).collect::<Vec<_>>().join("\n");

            let fixed_count = fixed_cnrs.len();
            let fixed = fixed_cnrs.iter().map(
                |(cnr_name, content_link, prev_error)| {
                    format!(r#"
<h4>{cnr_name}</h4>

<p><a href="../blobs/{content_link}.html">code</a></p>

<h5>Previous Error</h5>

<pre class="code-block"><code>
{prev_error}
</code></pre>
"#)
                }
            ).collect::<Vec<_>>().join("\n");

            let regressed_count = regressed_cnrs.len();
            let regressed = regressed_cnrs.iter().map(
                |(cnr_name, content_link, next_error)| {
                    format!(r#"
<h4>{cnr_name}</h4>

<p><a href="../blobs/{content_link}.html">code</a></p>

<h5>Error</h5>

<pre class="code-block"><code>
{next_error}
</code></pre>
"#)
                }
            ).collect::<Vec<_>>().join("\n");

            let different_errors_count = different_error_cnrs.len();
            let different_errors = different_error_cnrs.iter().map(
                |cnr| {
                    format!(r#""#)
                }
            ).collect::<Vec<_>>().join("\n");

            let removed_count = removed_cnrs.len();
            let removed = removed_cnrs.iter().map(
                |cnr| {
                    format!(r#""#)
                }
            ).collect::<Vec<_>>().join("\n");

            let added_count = added_cnrs.len();
            let added = added_cnrs.iter().map(
                |cnr| {
                    format!(r#""#)
                }
            ).collect::<Vec<_>>().join("\n");

            format!(r#"
<h3>Updated ({updated_count})</h3>

{updated}

<h3>Fixed ({fixed_count})</h3>

{fixed}

<h3>Regressed ({regressed_count})</h3>

{regressed}

<h3>Different Errors ({different_errors_count})</h3>

{different_errors}

<h3>Added ({added_count})</h3>

{added}

<h3>Removed ({removed_count})</h3>

{removed}
"#)
        },
        _ => String::from("<p>Cnr diff is not available!</p>"),
    };

    let cnr_diff = format!(r#"
<h1>{title}</h1>

<p>{link1}</p>
<p>{link2}</p>

<h2>Cnr Diff</h2>

{cnr_diff}
"#);

    html_template(&cnr_diff, true)
}

fn diff_blob(a: &[u8], b: &[u8]) -> String {
    write_bytes(
        "tmp-blob-a",
        a,
        WriteMode::CreateOrTruncate,
    ).unwrap();

    write_bytes(
        "tmp-blob-b",
        b,
        WriteMode::CreateOrTruncate,
    ).unwrap();

    let diff = subprocess::run(
        "git",
        &[
            "diff",
            "-U5",
            "--diff-algorithm=patience",
            "--color=never",
            "--no-index",
            "--",
            "tmp-blob-a",
            "tmp-blob-b",
        ],
        ".",
        5.0,
        false,

        // I don't know why, but it always ends with nonzero-status.
        false,
    ).unwrap();

    let s = String::from_utf8_lossy(&diff.stdout);
    let s = color_udiff(&s);

    remove_file("tmp-blob-a").unwrap();
    remove_file("tmp-blob-b").unwrap();

    s
}
