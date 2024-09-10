use crate::HirSession;
use sodigy_ast::IdentWithSpan;
use sodigy_session::SodigySession;

// This function is quite expensive.
// docs.map(remove_leading_whitespace).join("\n")
pub fn concat_doc_comments(
    docs: &Vec<IdentWithSpan>,
    session: &mut HirSession,
) -> Option<IdentWithSpan> {
    if docs.is_empty() {
        None
    }

    else {
        let first_span = docs[0].span();
        let last_span = docs.last().unwrap().span();

        let d = docs.iter().map(
            |d| {
                let d = session.unintern_string(d.id());

                remove_leading_whitespace(d)
            }
        ).collect::<Vec<String>>().join("\n");

        Some(IdentWithSpan::new(
            session.intern_string(d.into()),
            if docs.len() > 1 {
                first_span.merge(*last_span)
            } else {
                *first_span
            },
        ))
    }
}

fn remove_leading_whitespace(s: &[u8]) -> String {
    let mut index = 0;

    while s.get(index) == Some(&b' ') {
        index += 1;
    }

    String::from_utf8(s[index..].to_vec()).unwrap()
}
