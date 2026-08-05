use crate::html_template;
use sodigy_compiler_test::meta::git::CommitInfo;

pub fn render_commit(commit: &CommitInfo) -> String {
    let abbrev_hash = commit.commit_hash.get(0..9).unwrap().to_string();

    let title = &commit.title;
    let body = match &commit.body {
        Some(body) => format!(r#"<pre class="code-block"><code>{body}</code></pre>"#),
        None => String::new(),
    };
    let author = &commit.author;
    let author_email = &commit.author_email;
    let parent = match &commit.parent_hash {
        Some(parent) => {
            let abbrev_hash = parent.get(0..9).unwrap().to_string();
            format!(r#"<li><a href="{abbrev_hash}.html">parent</a></li>"#)
        },
        None => String::new(),
    };

    // TODO: `git diff -U5 --color=always --diff-algorithm=patience <hash1> <hash2>`

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
"#),
        true,
    )
}
