use crate::{escape_html, html_template};
use sodigy_compiler_test::meta::git::CommitInfo;
use sodigy_compiler_test::subprocess;

pub fn render_commit(commit: &CommitInfo) -> String {
    let abbrev_hash = commit.commit_hash.get(0..9).unwrap().to_string();

    let title = &commit.title;
    let body = match &commit.body {
        Some(body) => format!(r#"<pre class="code-block"><code>{body}</code></pre>"#),
        None => String::new(),
    };
    let author = &commit.author;
    let author_email = &commit.author_email;
    let (parent, diff) = match &commit.parent_hash {
        Some(parent) => {
            let parent_hash = parent.get(0..9).unwrap().to_string();
            (
                format!(r#"<li><a href="{parent_hash}.html">parent</a></li>"#),
                format!(
                    r#"<pre class="code-block"><code>{}</code></pre>"#,
                    escape_html(&diff_commits(&parent_hash, &abbrev_hash)),
                ),
            )
        },
        None => (String::new(), String::new()),
    };

    html_template(
        &format!(r#"
<h1>{abbrev_hash}</h1>

<h2>{title}</h2>

<ul>
    <li>author: {author}</li>
    <li>author_email: {author_email}</li>
    {parent}
</ul>

{body}

{diff}

"#),
        true,
    )
}

fn diff_commits(prev: &str, next: &str) -> String {
    let o = subprocess::run(
        "git",
        &[
            "diff",
            "-U5",
            "--diff-algorithm=patience",
            "--color=never",
            prev,
            next,
        ],
        ".",
        5.0,
        false,
        true,
    ).unwrap();
    let s = String::from_utf8_lossy(&o.stdout);
    let mut colored = vec![];

    for line in s.lines() {
        if line.starts_with("+") {
            colored.push(format!("\x1b[32m{line}\x1b[0m"));
        }

        else if line.starts_with("-") {
            colored.push(format!("\x1b[31m{line}\x1b[0m"));
        }

        else if line.starts_with("@") {
            colored.push(format!("\x1b[33m{line}\x1b[0m"));
        }

        else {
            colored.push(line.to_string());
        }
    }

    colored.join("\n")
}
