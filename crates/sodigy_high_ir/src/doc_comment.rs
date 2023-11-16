use crate::HirSession;
use sodigy_intern::InternedString;
use sodigy_span::SpanRange;

// This function is quite expensive.
// docs.map(remove_leading_whitespace).join("\n")
pub fn concat_doc_comments(
    docs: &Vec<(InternedString, SpanRange)>,
    session: &mut HirSession,
) -> Option<InternedString> {
    if docs.is_empty() {
        None
    }

    else {
        let d = docs.iter().map(
            |d| {
                let d = session.unintern_string(d.0).unwrap();

                remove_leading_whitespace(d)
            }
        ).collect::<Vec<String>>().join("\n");

        Some(session.intern_string(d.into()))
    }
}

fn remove_leading_whitespace(s: &[u8]) -> String {
    let mut ind = 0;

    while s.get(ind) == Some(&b' ') {
        ind += 1;
    }

    String::from_utf8(s[ind..].to_vec()).unwrap()
}
