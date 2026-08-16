use crate::{
    PolySpanKind,
    RenderableSpan,
    RenderSpanOption as Option,
    RenderSpanSession as Session,
    Span,
    SpanDeriveKind,
    render_spans,
};
use sodigy_file::File;
use sodigy_fs_api::{
    FileError,
    WriteMode,
    create_dir,
    exists,
    join,
    remove_dir_all,
    remove_file,
    write_bytes,
};
use std::mem::size_of;

#[test]
fn size_assertions() {
    assert!(size_of::<PolySpanKind>() <= 32, "{}", size_of::<PolySpanKind>());
    assert!(size_of::<SpanDeriveKind>() <= 32, "{}", size_of::<SpanDeriveKind>());
    // 32 bytes when mono_id is either u128 or u64
    assert!(size_of::<Span>() <= 32, "{}", size_of::<Span>());
}

#[test]
fn render_test() {
    let intermediate_dir = "span-test-target";
    let mut failures = vec![];
    let samples: Vec<(&[u8], &[(&[u8], &str)], &str)> = vec![
        (
            b"abcdefg",
            &[(b"cde", "note-1")],
            "
src: render-test-content:1:3
   1 | abcdefg
     |   ^^^
     |   |
     |   note-1
",
        ),
        (
            b"abcdefghijklmnopqrstuvwxyz",
            &[(b"cde", "This is a very very long note"), (b"hij", "note-2")],
            "
src: render-test-content:1:3
   1 | abcdefghijklmnopqrstuvwxyz
     |   ^^^  ^^^
     |   |    |
     |   |   (1)
     |   |
     |   *--(0)

(0): This is a very very long note
(1): note-2
",
        ),
    ];

    for (content, labels, answer) in samples {
        match render_test_worker(content, labels, answer, intermediate_dir) {
            Ok(Ok(())) => {},
            Ok(Err((answer, result))) => {
                failures.push((answer, result));
            },
            Err(e) => {
                if exists(intermediate_dir) {
                    remove_dir_all(intermediate_dir).unwrap();
                }

                if exists("render-test-content") {
                    remove_file("render-test-content").unwrap();
                }

                panic!("{e:?}");
            },
        }
    }

    if !failures.is_empty() {
        panic!(
            "{}",
            failures.iter().map(
                |(answer, result)| format!("--- answer ---\n{answer}\n--- result ---\n{result}")
            ).collect::<Vec<_>>().join("\n-------------------------\n"),
        )
    }
}

fn render_test_worker(
    content: &[u8],
    labels: &[(&[u8], &str)],
    answer: &str,
    intermediate_dir: &str,
) -> Result<Result<(), (String, String)>, FileError> {
    if exists(intermediate_dir) {
        remove_dir_all(intermediate_dir)?;
    }

    create_dir(intermediate_dir)?;
    create_dir(&join(intermediate_dir, "str")?)?;
    create_dir(&join(intermediate_dir, "file_map")?)?;

    write_bytes(
        "render-test-content",
        content,
        WriteMode::CreateOrTruncate,
    )?;

    let file = File::register("render-test-content", "render-test-content", intermediate_dir)?;
    let mut spans = vec![];

    for i in 0..content.len() {
        for (label, note) in labels.iter() {
            if label[0] == content[i] && i + label.len() < content.len() && &content[i..(i + label.len())] == *label {
                spans.push(RenderableSpan {
                    span: Span::range(file, i as u32, label.len() as u32),
                    auxiliary: true,
                    note: Some(note.to_string()),
                });
            }
        }
    }

    let mut session = Session::new(intermediate_dir);
    let option = Option {
        max_height: 20,
        max_width: 200,
        context: 3,
        render_source: true,
        color: None,
        group_delim: None,
    };

    let result = render_spans(
        &spans,
        &option,
        &mut session,
    );

    remove_file("render-test-content")?;
    remove_dir_all(intermediate_dir)?;

    if result.trim().replace(" ", "") != answer.trim().replace(" ", "") {
        Ok(Err((answer.to_string(), result.to_string())))
    }

    else {
        Ok(Ok(()))
    }
}
