use crate::{color_udiff, escape_html, html_template, render_toc};
use sodigy_compiler_test::{
    CompileAndRun,
    CrateTest,
    TestHarness,
    diff_strings,
};
use sodigy_fs_api::set_extension;
use std::collections::{HashMap, HashSet};

pub fn render_diff(
    prev: &TestHarness,
    next: &TestHarness,
    blobs: &HashMap<String, Vec<u8>>,
) -> String {
    let crate_diff = match (prev, next) {
        (
            TestHarness { crates: Some(prev), .. },
            TestHarness { crates: Some(next), .. },
        ) => {
            let prev_by_name: HashMap<String, &CrateTest> = prev.iter().map(
                |c| (c.name.to_string(), c)
            ).collect();
            let next_by_name: HashMap<String, &CrateTest> = next.iter().map(
                |c| (c.name.to_string(), c)
            ).collect();
            let mut all_names: Vec<String> = prev.iter().chain(next.iter()).map(
                |c| c.name.to_string()
            ).collect::<HashSet<_>>().into_iter().collect();
            all_names.sort();

            // Vec<(crate_name, prev_summary)>
            let mut fixed_crates: Vec<(String, String)> = vec![];

            // Vec<(crate_name, next_summary)>
            let mut regressed_crates: Vec<(String, String)> = vec![];

            // Vec<(crate_name, prev_summary, next_summary)>
            let mut different_error_crates: Vec<(String, String, String)> = vec![];

            // Vec<crate_name>
            let mut added_crates: Vec<String> = vec![];

            // Vec<crate_name>
            let mut removed_crates: Vec<String> = vec![];

            for crate_name in all_names.iter() {
                match (prev_by_name.get(crate_name), next_by_name.get(crate_name)) {
                    (Some(prev), Some(next)) => {
                        let prev_summary = prev.summary();
                        let next_summary = next.summary();

                        match (prev.has_error(), next.has_error()) {
                            (true, true) if prev_summary != next_summary => {
                                different_error_crates.push((crate_name.to_string(), prev_summary, next_summary));
                            },
                            (true, false) => {
                                fixed_crates.push((crate_name.to_string(), prev_summary));
                            },
                            (false, true) => {
                                regressed_crates.push((crate_name.to_string(), next_summary));
                            },
                            _ => {},
                        }
                    },
                    (Some(_), None) => {
                        removed_crates.push(crate_name.to_string());
                    },
                    (None, Some(_)) => {
                        added_crates.push(crate_name.to_string());
                    },
                    (None, None) => unreachable!(),
                }
            }

            let fixed_count = fixed_crates.len();
            let fixed = fixed_crates.iter().map(
                |(crate_name, prev_summary)| {
                    format!(r#"
<h4>{crate_name}</h4>

<h5>Previous Error</h5>

<pre class="code-block"><code>{prev_summary}</code></pre>
"#)
                }
            ).collect::<Vec<_>>().join("\n");

            let regressed_count = regressed_crates.len();
            let regressed = regressed_crates.iter().map(
                |(crate_name, next_summary)| {
                    format!(r#"
<h4>{crate_name}</h4>

<h5>Current Error</h5>

<pre class="code-block"><code>{next_summary}</code></pre>
"#)
                }
            ).collect::<Vec<_>>().join("\n");

            let different_errors_count = different_error_crates.len();
            let different_errors = different_error_crates.iter().map(
                |(crate_name, prev_summary, next_summary)| {
                    format!(r#"
<h4>{crate_name}</h4>

<h5>Previous Error</h5>

<pre class="code-block"><code>{prev_summary}</code></pre>

<h5>Current Error</h5>

<pre class="code-block"><code>{next_summary}</code></pre>
"#)
                }
            ).collect::<Vec<_>>().join("\n");

            let added_count = added_crates.len();
            let added = added_crates.iter().map(
                |crate_name| format!("<h4>{crate_name}</h4>")
            ).collect::<Vec<_>>().join("\n");

            let removed_count = removed_crates.len();
            let removed = removed_crates.iter().map(
                |crate_name| format!("<h4>{crate_name}</h4>")
            ).collect::<Vec<_>>().join("\n");

            let toc = render_toc(vec![
                (String::from("Fixed"), String::from("crt-fixed"), Some(fixed_count.to_string()), None),
                (String::from("Regressed"), String::from("crt-regressed"), Some(regressed_count.to_string()), None),
                (String::from("Different Errors"), String::from("crt-different-errors"), Some(different_errors_count.to_string()), None),
                (String::from("Added"), String::from("crt-added"), Some(added_count.to_string()), None),
                (String::from("Removed"), String::from("crt-removed"), Some(removed_count.to_string()), None),
            ]);

            format!(r#"
{toc}

<h3 id="crt-fixed">Fixed ({fixed_count})</h3>

{fixed}

<h3 id="crt-regressed">Regressed ({regressed_count})</h3>

{regressed}

<h3 id="crt-different-errors">Different Errors ({different_errors_count})</h3>

{different_errors}

<h3 id="crt-added">Added ({added_count})</h3>

{added}

<h3 id="crt-removed">Removed ({removed_count})</h3>

{removed}
"#)
        },
        _ => String::from("<p>Crate diff is not available!</p>"),
    };

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
                |(cnr_name, content_link, prev_error, next_error)| {
                    format!(r#"
<h4>{cnr_name}</h4>

<p><a href="../blobs/{content_link}.html">code</a></p>

<h5>Old Error</h5>

<pre class="code-block"><code>
{prev_error}
</code></pre>

<h5>New Error</h5>

<pre class="code-block"><code>
{next_error}
</code></pre>
"#)
                }
            ).collect::<Vec<_>>().join("\n");

            let added_count = added_cnrs.len();
            let added = added_cnrs.iter().map(
                |(cnr_name, content_link)| {
                    format!(r#"
<h4>{cnr_name}</h4>

<p><a href="../blobs/{content_link}.html">code</a></p>
"#)
                }
            ).collect::<Vec<_>>().join("\n");

            let removed_count = removed_cnrs.len();
            let removed = removed_cnrs.iter().map(
                |(cnr_name, content_link)| {
                    format!(r#"
<h4>{cnr_name}</h4>

<p><a href="../blobs/{content_link}.html">code</a></p>
"#)
                }
            ).collect::<Vec<_>>().join("\n");

            let toc = render_toc(vec![
                (String::from("Updated"), String::from("cnr-updated"), Some(updated_count.to_string()), None),
                (String::from("Fixed"), String::from("cnr-fixed"), Some(fixed_count.to_string()), None),
                (String::from("Regressed"), String::from("cnr-regressed"), Some(regressed_count.to_string()), None),
                (String::from("Different Errors"), String::from("cnr-different-errors"), Some(different_errors_count.to_string()), None),
                (String::from("Added"), String::from("cnr-added"), Some(added_count.to_string()), None),
                (String::from("Removed"), String::from("cnr-removed"), Some(removed_count.to_string()), None),
            ]);

            format!(r#"
{toc}

<h3 id="cnr-updated">Updated ({updated_count})</h3>

{updated}

<h3 id="cnr-fixed">Fixed ({fixed_count})</h3>

{fixed}

<h3 id="cnr-regressed">Regressed ({regressed_count})</h3>

{regressed}

<h3 id="cnr-different-errors">Different Errors ({different_errors_count})</h3>

{different_errors}

<h3 id="cnr-added">Added ({added_count})</h3>

{added}

<h3 id="cnr-removed">Removed ({removed_count})</h3>

{removed}
"#)
        },
        _ => String::from("<p>Cnr diff is not available!</p>"),
    };

    let file1 = prev.meta.get_result_file_name();
    let file2 = next.meta.get_result_file_name();
    let title = format!("{file1} vs {file2}");
    let link1 = format!(r#"<a href="../harnesses/{}">{file1}</a>"#, set_extension(&file1, "html").unwrap());
    let link2 = format!(r#"<a href="../harnesses/{}">{file2}</a>"#, set_extension(&file2, "html").unwrap());
    let html = format!(r#"
<h1>{title}</h1>

<p>old: {link1}</p>
<p>new: {link2}</p>

<h2>Crate Diff</h2>

{crate_diff}

<h2>Cnr Diff</h2>

{cnr_diff}
"#);

    html_template(&html, true)
}

fn diff_blob(a: &[u8], b: &[u8]) -> String {
    let a = String::from_utf8_lossy(a);
    let b = String::from_utf8_lossy(b);
    color_udiff(&diff_strings(&a, &b))
}
