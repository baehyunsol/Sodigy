use crate::{
    circle,
    escape_html,
    html_template,
    render_elapsed_ms,
    render_toc,
};
use sodigy_compiler_test::{CrateTestResult, TestHarness};
use sodigy_fs_api::set_extension;

pub fn render_harness(
    harness: &TestHarness,
    prev: Option<String>,
    next: Option<String>,
) -> String {
    let toc = render_toc(vec![
        (
            String::from("Crate tests"),
            String::from("tt-crates"),
            harness.crates.as_ref().map(
                |crates| format!(
                    "{}/{}",
                    crates.iter().filter(|c| !c.has_error()).count(),
                    crates.len(),
                )
            ),
            harness.crates.as_ref().map(
                |crates| crates.iter().filter(|c| c.has_error()).count() == 0
            ),
        ),
        (
            String::from("Compile-And-Run"),
            String::from("tt-cnr"),
            harness.compile_and_run.as_ref().map(
                |cnrs| format!(
                    "{}/{}",
                    cnrs.iter().filter(|cnr| cnr.error.is_none()).count(),
                    cnrs.len(),
                )
            ),
            harness.compile_and_run.as_ref().map(
                |cnrs| cnrs.iter().filter(|cnr| cnr.error.is_some()).count() == 0
            ),
        ),
        (
            String::from("Fuzz"),
            String::from("tt-fuzz"),
            None,
            harness.fuzz.as_ref().map(
                |fuzz| fuzz.iter().filter(|f| f.artifact.is_some()).count() == 0
            ),
        ),
    ]);

    let meta = {
        let commit_hash = &harness.meta.commit.commit_hash;
        let cargo_version = &harness.meta.cargo_version;
        let rustc_version = &harness.meta.rustc_version;
        let os = format!("{:?}", harness.meta.os).to_ascii_lowercase();
        let started_at = &harness.meta.started_at;

        format!(r#"
<pre class="code-block"><code>
- commit
  - hash: {commit_hash}
  - <a href="../commits/{commit_hash}.html">more info</a>
- cargo version: {cargo_version}
- rustc version: {rustc_version}
- os: {os}
- started at: {started_at}
</code></pre>
"#)
    };

    let crates = if let Some(crates) = &harness.crates && !crates.is_empty() {
        let summary = format!(
            "{}/{}",
            crates.iter().filter(|c| !c.has_error()).count(),
            crates.len(),
        );

        let toc = render_toc(crates.iter().map(
            |c| (
                c.name.to_string(),
                format!("crt-{}", c.name),
                None,
                Some(!c.has_error()),
            )
        ).collect());

        let crates = crates.iter().map(
            |c| {
                fn each_crate(title: &str, result: &CrateTestResult) -> String {
                    fn color_warnings_and_errors(e: &str) -> String {
                        let mut result = vec![];

                        for line in e.lines() {
                            if line.starts_with("error: ") {
                                result.push(format!("\x1b[31merror: \x1b[0m{}", line.get(7..).unwrap()));
                            }

                            else if line.starts_with("warning: ") {
                                result.push(format!("\x1b[33mwarning: \x1b[0m{}", line.get(9..).unwrap()));
                            }

                            else {
                                result.push(line.to_string());
                            }
                        }

                        result.join("\n")
                    }

                    let elapsed_time = render_elapsed_ms(result.elapsed_ms);
                    let result = match &result.error {
                        Some(error) => {
                            let error = escape_html(&color_warnings_and_errors(error));
                            format!(r#"
<details>
    <summary><span class="red">stderr</span></summary>
    <pre class="code-block"><code>{error}</code></pre>
</details>
"#)
                        },
                        None => String::from("Successful"),
                    };

                    format!(r#"
<h4><code class="code-span">{title}</code></h4>

<p>Elapsed: {elapsed_time}</p>
<p>{result}</p>
"#)
                }

                let name = &c.name;
                let marker = if c.has_error() {
                    circle("red", "medium")
                } else {
                    circle("green", "medium")
                };
                let clippy = each_crate("cargo clippy", &c.clippy);
                let doc = each_crate("cargo doc", &c.doc);
                let debug = each_crate("cargo test", &c.debug);
                let release = each_crate("cargo test --release", &c.release);

                format!(r#"
<h3 id="crt-{name}">{name} {marker}</h3>

{clippy}

{doc}

{debug}

{release}
"#)
            }
        ).collect::<Vec<_>>().join("\n");

        format!(r#"
<h2 id="tt-crates">Crate tests ({summary})</h2>

{toc}

{crates}
"#)
    } else {
        String::from(r#"
<h2 id="tt-crates">Crate tests</h2>

<p>N/A</p>
"#)
    };

    let cnrs = if let Some(cnrs) = &harness.compile_and_run && !cnrs.is_empty() {
        let summary = format!(
            "{}/{}",
            cnrs.iter().filter(|cnr| cnr.error.is_none()).count(),
            cnrs.len(),
        );

        let toc = render_toc(cnrs.iter().map(
            |cnr| (
                cnr.name.to_string(),
                format!("cnr-{}", cnr.name),
                None,
                Some(cnr.error.is_none()),
            )
        ).collect());

        let cnrs = cnrs.iter().map(
            |cnr| {
                let name = &cnr.name;
                let marker = if cnr.error.is_some() {
                    circle("red", "medium")
                } else {
                    circle("green", "medium")
                };
                let code = format!(r#"
<a href="../blobs/{}.html">code</a>
"#,
                    cnr.hash,
                );
                let error = match &cnr.error {
                    Some(error) => format!(r#"
<details>
    <summary><span class="red">error</span></summary>
    <pre class="code-block"><code>{}</code></pre>
</details>
"#,
                        escape_html(error),
                    ),
                    None => String::new(),
                };
                let stdout = escape_html(&cnr.stdout_colored);
                let stderr = escape_html(&cnr.stderr_colored);
                let compile_elapsed = render_elapsed_ms(cnr.compile_elapsed_ms);
                let run_elapsed = match cnr.run_elapsed_ms {
                    Some(ms) => render_elapsed_ms(ms),
                    None => String::from("N/A"),
                };

                format!(r#"
<h3 id="cnr-{name}">{name} {marker}</h3>

{code}

<p>compile: {compile_elapsed}</p>
<p>run: {run_elapsed}</p>

{error}

<details>
    <summary>stdout</summary>
    <pre class="code-block"><code>{stdout}</code></pre>
</details>

<details>
    <summary>stderr</summary>
    <pre class="code-block"><code>{stderr}</code></pre>
</details>

"#)
            }
        ).collect::<Vec<_>>().join("\n");

        format!(r#"
<h2 id="tt-cnr">Compile-And-Run tests ({summary})</h2>

{toc}

{cnrs}
"#)
    } else {
        String::from(r#"
<h2 id="tt-cnr">Compile-And-Run tests</h2>

<p>N/A</p>
"#)
    };

    let fuzz = if let Some(fuzz) = &harness.fuzz && !fuzz.is_empty() {
        let summary = format!(
            "{}/{}",
            fuzz.iter().filter(|fuzz| fuzz.artifact.is_none()).count(),
            fuzz.len(),
        );

        let toc = render_toc(fuzz.iter().map(
            |f| (
                f.target.name().to_string(),
                format!("fuzz-{}", f.target.name()),
                None,
                Some(f.artifact.is_none()),
            )
        ).collect());

        let fuzz = fuzz.iter().map(
            |f| {
                let name = f.target.name();
                let elapsed = render_elapsed_ms(f.elapsed_ms);
                let result = match &f.artifact {
                    Some(artifact) => format!(r#"
<details>
    <summary>artifact</summary>
    <pre class="code-block"><code>{}</code></pre>
</details>
"#,
                        String::from_utf8_lossy(artifact),
                    ),
                    None => String::new(),
                };

                format!(r#"
<h3 id="fuzz-{name}">{name}</h3>

<p>elapsed: {elapsed}</p>

<p>{result}</p>
"#)
            }
        ).collect::<Vec<_>>().join("\n");

        format!(r#"
<h2 id="tt-fuzz">Fuzz ({summary})</h2>

{toc}

{fuzz}
"#)
    } else {
        String::from(r#"
<h2 id="tt-cnr">Compile-And-Run tests</h2>

<p>N/A</p>
"#)
    };

    let curr_commit_hash = harness.meta.commit.commit_hash.get(0..9).unwrap();
    let title = harness.meta.get_result_file_name();
    let prev = render_nav(curr_commit_hash, prev, "<< prev");
    let next = render_nav(curr_commit_hash, next, "next >>");

    html_template(
        &format!(
r#"
<h1>{title}</h1>

<p>
    {prev}
    {next}
</p>

{meta}

{toc}

{crates}

{cnrs}

{fuzz}
"#,
        ),
        true,
    )
}

fn render_nav(
    src: &str,
    dst: Option<String>,
    title: &str,
) -> String {
    let diff = match &dst {
        Some(dst) => {
            let dst = dst.get(12..21).unwrap();
            let src = u64::from_str_radix(src, 16).unwrap();
            let dst = u64::from_str_radix(dst, 16).unwrap();
            let diff_hash = format!("{:09x}", src ^ dst);

            format!(r#"<a href="../diffs/{diff_hash}.html">diff</a>"#)
        },
        None => String::from("diff: N/A"),
    };
    let dst = match &dst {
        Some(dst) => format!(r#"<a href="{}">{dst}</a>"#, set_extension(dst, "html").unwrap()),
        None => String::from("N/A"),
    };

    let title = escape_html(title);

    format!(r#"
<div class="nav-box center-text">
    <p>{title}</p>
    <p>{dst}</p>
    <p>{diff}</p>
</div>
"#)
}
