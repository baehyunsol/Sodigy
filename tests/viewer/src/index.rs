use crate::html_template;
use sodigy_compiler_test::TestHarness;
use sodigy_compiler_test::meta::git::CommitInfo;
use sodigy_fs_api::{file_name, set_extension};
use std::collections::HashMap;

pub fn render_index(
    harnesses: &HashMap<String, Vec<String>>,
    harnesses_by_name: &HashMap<String, TestHarness>,
    commits: &[CommitInfo],
) -> String {
    let commits = commits.iter().map(
        |commit| {
            let abbrev_hash = commit.commit_hash.get(0..9).unwrap().to_string();
            let data = match harnesses.get(&abbrev_hash) {
                Some(harnesses) if !harnesses.is_empty() => format!(
                    "<ul>{}</ul>",
                    harnesses.iter().map(
                        |h| {
                            let harness = harnesses_by_name.get(h).unwrap();
                            let url = set_extension(h, "html").unwrap();
                            let base = file_name(h).unwrap();
                            format!(
                                r#"<li><a href="harnesses/{url}">{base}</a> ({})</li>"#,
                                harness.meta.started_at,
                            )
                        }
                    ).collect::<Vec<_>>().concat(),
                ),
                _ => String::new(),
            };

            format!(
                r#"<li><a href=commits/{abbrev_hash}.html>{abbrev_hash}</a>{data}</li>"#,
            )
        }
    ).collect::<Vec<_>>().join("\n");

    html_template(
        &format!(r#"
<ul>
{commits}
</ul>
"#),
        false,
    )
}
