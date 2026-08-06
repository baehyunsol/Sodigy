use crate::html_template;
use chrono::DateTime;
use sodigy_compiler_test::meta::git::CommitInfo;
use sodigy_fs_api::{file_name, set_extension};
use std::collections::HashMap;

pub fn render_index(harnesses: &HashMap<String, Vec<String>>, commits: &[CommitInfo]) -> String {
    let commits = commits.iter().map(
        |commit| {
            let abbrev_hash = commit.commit_hash.get(0..9).unwrap().to_string();
            let title = &commit.title;

            let title = if title.chars().count() > 64 {
                format!("{}...", title.chars().take(61).collect::<String>())
            } else {
                format!("{title}{}", " ".repeat(64 - title.chars().count()))
            };

            // TODO: use `commit.timezone`
            let timestamp = DateTime::from_timestamp(commit.timestamp, 0).unwrap();
            let timestamp = timestamp.to_rfc3339();

            let data = match harnesses.get(&abbrev_hash) {
                Some(harnesses) if !harnesses.is_empty() => format!(
                    "<ul>{}</ul>",
                    harnesses.iter().map(
                        |h| {
                            let url = set_extension(h, "html").unwrap();
                            let base = file_name(h).unwrap();
                            format!(r#"<li><a href="harnesses/{url}">{base}</a></li>"#)
                        }
                    ).collect::<Vec<_>>().concat(),
                ),
                _ => String::new(),
            };

            format!(
                r#"<li><a class="monospace-font" href=commits/{abbrev_hash}.html>{abbrev_hash}</a> <code class="code-span">{title}</code> ({timestamp}) {data}</li>"#,
            )
        }
    ).collect::<Vec<_>>().join("\n");

    html_template(
        &format!(r#"
<ul class="commit-index">
{commits}
</ul>
"#),
        false,
    )
}
