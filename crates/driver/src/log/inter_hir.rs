use super::{
    FuncCall,
    create_preview_map,
    render_map_and_save,
    render_page_and_save,
};
use sodigy_fs_api::{FileError, join4};
use sodigy_inter_hir::{LogEntry, Session as InterHirSession};
use sodigy_span::{
    Color,
    ColorOption,
    RenderSpanOption,
    RenderSpanSession,
};

pub fn dump_inter_hir_log(session: &InterHirSession) -> Result<(), FileError> {
    fn to_func_call(log: &[LogEntry], mut index: usize, session: &InterHirSession) -> (FuncCall, usize) {
        let mut spans = vec![];
        let mut input = vec![];

        let call_index = index;
        let (name, log_id) = match &log[index] {
            _ => todo!(),
        };

        let mut children = vec![];
        index += 1;

        while log[index].id() != log_id {
            let (child, new_index) = to_func_call(log, index, session);
            index = new_index;
            children.push(child);
        }

        let mut output = vec![];
        let (has_error, last_errors) = match &log[index] {
            _ => todo!(),
        };
        let last_errors = last_errors.into_iter().map(|error| (None, error)).collect();
        let has_inner_error = has_error || children.iter().any(|c| c.has_inner_error);

        (
            FuncCall {
                call_index,
                name: name.to_string(),
                input,
                children,
                output,
                spans,
                has_error,
                last_errors,
                has_inner_error,
            },
            index + 1,
        )
    }

    let mut index = 0;
    let mut calls = vec![];

    while session.log.get(index).is_some() {
        let (call, new_index) = to_func_call(&session.log, index, session);
        index = new_index;
        calls.push(call);
    }

    let render_span_option = RenderSpanOption {
        max_height: 20,
        max_width: 160,
        context: 8,
        render_source: true,
        color: Some(ColorOption {
            primary: Color::Yellow,
            auxiliary: Color::Yellow,
            info: Color::Green,
        }),
        group_delim: None,
    };
    let mut render_span_session = RenderSpanSession::new(&session.intermediate_dir);
    let preview_map = create_preview_map(&calls);
    let log_dir = join4(
        &session.intermediate_dir,
        "irs",
        "interhir",
        "log",
    )?;

    for (i, _) in calls.iter().enumerate() {
        render_page_and_save(
            None,
            &calls,
            i,
            &preview_map,
            &log_dir,
            &session.intermediate_dir,
            &render_span_option,
            &mut render_span_session,
        )?;
    }

    render_map_and_save(&calls, &log_dir)?;
    Ok(())
}
