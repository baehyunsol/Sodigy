use sodigy_fs_api::{
    FileError,
    WriteMode,
    create_dir,
    exists,
    join4,
    parent,
    write_bytes,
    write_string,
};
use sodigy_post_mir::MatchDump;
use sodigy_span::{
    RenderSpanOption,
    RenderSpanSession,
    RenderableSpan,
    render_spans,
};

// TODO: it should dump html files, like the others!!
pub fn dump_post_mir_log(matches: &Vec<MatchDump>, intermediate_dir: &str) -> Result<(), FileError> {
    let mut buffer = vec![];
    let mut render_span_session = RenderSpanSession::new(intermediate_dir);
    let render_span_option = RenderSpanOption {
        max_width: 128,
        max_height: 20,
        context: 3,
        render_source: true,
        color: None,
        group_delim: None,
    };

    for MatchDump { keyword_span, span_helpers, decision_tree, expr } in matches.iter() {
        let mut spans = keyword_span.simple_error();

        for (span, helper) in span_helpers.iter() {
            spans.push(RenderableSpan {
                span: span.clone(),
                auxiliary: true,
                note: Some(helper.to_string()),
            });
        }

        buffer.push(String::from("------\n"));
        buffer.push(String::from("# Sodigy\n"));
        buffer.push(String::from("```\n"));
        buffer.push(render_spans(
            &spans,
            &render_span_option,
            &mut render_span_session,
        ));
        buffer.push(String::from("```\n"));
        buffer.push(String::new());
        buffer.push(String::from("# Decision Tree\n"));
        buffer.push(String::from("```\n"));
        buffer.push(decision_tree.to_string());
        buffer.push(String::from("```\n"));
        buffer.push(String::new());
        buffer.push(String::from("# Expr\n"));
        buffer.push(String::from("```\n"));
        buffer.push(expr.to_string());
        buffer.push(String::from("```\n"));
    }

    let save_at = join4(
        intermediate_dir,
        "irs",
        "postmir",
        "log",
    )?;

    if !exists(&parent(&save_at)?) {
        create_dir(&parent(&save_at)?)?;
    }

    if !exists(&save_at) {
        write_bytes(&save_at, b"", WriteMode::AlwaysCreate)?;
    }

    write_string(
        &save_at,
        &buffer.join("\n"),
        WriteMode::AlwaysAppend,
    )
}
