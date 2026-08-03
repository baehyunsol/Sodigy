use sodigy_compiler_test::{CrateTestResult, TestHarness};

// renders an html and returns the result
pub fn single_harness(harness: &TestHarness) -> String {
    let meta = {
        let commit_hash = &harness.meta.commit.commit_hash;
        let cargo_version = &harness.meta.cargo_version;
        let rustc_version = &harness.meta.rustc_version;
        let os = format!("{:?}", harness.meta.os).to_ascii_lowercase();
        let started_at = &harness.meta.started_at;

        format!(r#"
<pre class="meta"><code>
- commit
  - hash: {commit_hash}
  - <a href="../commit/{commit_hash}.html">more info</a>
- cargo version: {cargo_version}
- rustc version: {rustc_version}
- os: {os}
- started at: {started_at}
</code></pre>
"#)
    };

    fn render_toc(list: Vec<(String, String, Option<bool>)>) -> String {
        format!(r#"
<div class="toc">
<ol>
{}
</ol>
</div>
"#,
            list.iter().map(
                |(title, anchor, success)| format!(
                    r##"<li><a href="#{anchor}">{title}</a> {}</li>"##,
                    match success {
                        Some(true) => circle("green", "small"),
                        Some(false) => circle("red", "small"),
                        None => circle("white", "small"),
                    },
                )
            ).collect::<Vec<_>>().join("\n"),
        )
    }

    let crates = if let Some(crates) = &harness.crates && !crates.is_empty() {
        let toc = render_toc(crates.iter().map(
            |c| (
                c.name.to_string(),
                format!("crt-{}", c.name),
                Some(!c.has_error()),
            )
        ).collect());

        let crates = crates.iter().map(
            |c| {
                fn each_crate(title: &str, result: &CrateTestResult) -> String {
                    let elapsed_time = render_elapsed_ms(result.elapsed_ms);
                    let result = match &result.error {
                        Some(error) => format!("<pre><code>{}</code></pre>", render_ansi_term(error)),
                        None => String::from("Successful"),
                    };

                    format!(r#"
<h4><code class="code-span">{title}</code></h4>

<p>Elapsed: {elapsed_time}</p>
<p>{result}</p>
"#)
                }

                let name = &c.name;
                let clippy = each_crate("cargo clippy", &c.clippy);
                let doc = each_crate("cargo doc", &c.clippy);
                let debug = each_crate("cargo test", &c.clippy);
                let release = each_crate("cargo test --release", &c.clippy);

                format!(r#"
<h3 id="crt-{name}">{name}</h3>

{clippy}

{doc}

{debug}

{release}
"#)
            }
        ).collect::<Vec<_>>().join("\n");

        format!(r#"
<h2>Crate tests</h2>

{toc}

{crates}
"#)
    } else {
        String::from(r#"
<h2>Crate tests</h2>

N/A
"#)
    };

    let cnrs = if let Some(cnrs) = &harness.compile_and_run && !cnrs.is_empty() {
        let toc = render_toc(cnrs.iter().map(
            |cnr| (
                cnr.name.to_string(),
                format!("cnr-{}", cnr.name),
                Some(cnr.error.is_some()),
            )
        ).collect());

        let cnrs = cnrs.iter().map(
            |cnr| {
                let name = &cnr.name;
                let stdout = render_ansi_term(&cnr.stdout_colored);
                let stderr = render_ansi_term(&cnr.stderr_colored);
                let compile_elapsed = render_elapsed_ms(cnr.compile_elapsed_ms);
                let run_elapsed = match cnr.run_elapsed_ms {
                    Some(ms) => render_elapsed_ms(ms),
                    None => String::from("N/A"),
                };

format!(r#"
<h3 id="cnr-{name}">{name}</h3>

<p>compile: {compile_elapsed}</p>
<p>run: {run_elapsed}</p>

<h4>stdout</h4>

<pre><code>{stdout}</code></pre>

<h4>stderr</h4>

<pre><code>{stderr}</code></pre>
"#)
            }
        ).collect::<Vec<_>>().join("\n");

        format!(r#"
<h2>Compile-And-Run tests</h2>

{toc}

{cnrs}
"#)
    } else {
        String::from(r#"
<h2>Compile-And-Run tests</h2>

N/A
"#)
    };

    let title = harness.meta.get_result_file_name();

    html_template(
        &format!(
r#"
<h1>{title}</h1>

{meta}

{crates}

{cnrs}
"#,
        ))
}

fn circle(color: &str, size: &str) -> String {
    format!(r#"<span class="circle-{color} {size}"></span>"#)
}

fn render_elapsed_ms(ms: u64) -> String {
    match ms {
        0..1000 => format!("0.{ms:03}s"),
        1_000..20_000 => format!("{}.{:03}s", ms / 1000, ms % 1000),
        20_000..60_000 => format!("{}s", ms / 1000),
        60_000.. => format!("{}m {}s", ms / 60_000, ms / 1_000 % 60),
    }
}

fn render_ansi_term(c: &str) -> String {
    c.to_string()  // TODO
}

const STYLE: &str = include_str!("style.css");

fn html_template(body: &str) -> String {
    format!(r#"
<!DOCTYPE html>
<html>

<head>
<style>
{STYLE}
</style>
</head>

<body>
{body}
</body>

</html>
"#)
}
