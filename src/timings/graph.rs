use super::{Frame, Row, TimingsEntry, render_micro_seconds};

const ERROR_MARK: &'static str = r#"<span class="color-red error-mark">!</span>"#;

pub fn render_graph(id: &str, rows: &[Row], start: u64, end: u64, canvas_size: usize) -> String {
    let frame_count = rows[0].frames.len();
    let x_labels = {
        let mut labels = vec![];
        let label_count = (canvas_size / 160).max(4);

        for i in 1..label_count {
            let label = i as u64 * (end - start) / label_count as u64 + start;
            let label = render_micro_seconds(label);
            let left = i * canvas_size / label_count;
            labels.push(format!(r#"<span class="x-label" style="left: {}px;">{label}</span>"#, left - 30));
            labels.push(format!(r#"<span class="x-label-marker" style="left: {left}px;"></span>"#));
        }

        labels.concat()
    };

    let y_labels = {
        let mut y_labels = vec![];

        // empty label for x-labels
        y_labels.push(String::from(r#"<div class="graph-row graph-row-label"></div>"#));

        for row in rows.iter() {
            y_labels.push(format!(
                r#"<div class="graph-row graph-row-label">{}{}</div>"#,
                row.id,
                if row.has_error { ERROR_MARK } else { "" },
            ));
        }

        y_labels.concat()
    };

    let rendered_rows = {
        let mut rendered_rows = vec![];
        rendered_rows.push(format!(r#"<div class="graph-row graph-row-blocks"><span id="x-labels">{x_labels}</span></div>"#));

        for row in rows.iter() {
            let mut curr_block = None;
            let mut blocks = vec![];

            for (i, frame) in row.frames.iter().enumerate() {
                match frame {
                    Frame::New(_) | Frame::Empty => {
                        if let Some(block) = curr_block {
                            blocks.push(generate_block(&block, i, frame_count, canvas_size));
                        }

                        curr_block = None;

                        if let Frame::New(e) = frame {
                            curr_block = Some((e, i));
                        }
                    },
                    Frame::Same => {},
                }
            }

            if let Some(block) = curr_block {
                blocks.push(generate_block(&block, frame_count, frame_count, canvas_size));
            }

            let blocks = blocks.concat();
            rendered_rows.push(format!(r#"<div class="graph-row graph-row-blocks" style="width: {canvas_size}px;">{blocks}</div>"#));
        }

        rendered_rows.concat()
    };

    format!(r#"
<div id="{id}" class="graph">
    <div class="graph-canvas">
        <div class="graph-labels-column graph-column">{y_labels}</div>
        <div class="graph-rows-column graph-column">
            <div id="graph-rows-wrapper">{rendered_rows}</div>
        </div>
    </div>
</div>"#)
}

fn generate_block(
    (entry, start): &(&TimingsEntry, usize),
    end: usize,
    frame_count: usize,
    canvas_size: usize,
) -> String {
    let tooltip_message = format!(
        "{:?}{}<br/>({:.2}ms){}",
        entry.stage,
        if let Some(module) = &entry.module { format!("<br/>{module}") } else { String::new() },
        (entry.end - entry.start) as f64 / 1000.0,
        if entry.has_error { r#"<br/><span class="color-red">has error</span>"# } else { "" },
    );
    let tooltip_style = if (*start + end) < frame_count / 64 {
        // right-align (default)
        ""
    } else {
        // center-align
        "transform: translateX(-50%);"
    };

    let tooltip_container = format!(r#"<span class="tooltip" style="{tooltip_style}">{tooltip_message}</span>"#);

    let width = (end - start) * canvas_size / frame_count;
    let left = start * canvas_size / frame_count;
    let long_title = format!(
        "{:?}{}",
        entry.stage,
        if let Some(module) = &entry.module { format!(" ({module})") } else { String::new() },
    );
    let long_title_len = long_title.len() + if entry.has_error { 3 } else { 0 };
    let long_title_with_error_mark = format!(
        "{long_title}{}",
        if entry.has_error { ERROR_MARK } else { "" },
    );
    let short_title = format!("{:?}", entry.stage);
    let short_title_len = short_title.len() + if entry.has_error { 3 } else { 0 };
    let short_title_with_error_mark = format!(
        "{short_title}{}",
        if entry.has_error { ERROR_MARK } else { "" },
    );

    let title = if width > long_title_len * 8 {
        long_title_with_error_mark
    } else if width > short_title_len * 8 {
        short_title_with_error_mark
    } else if width > 20 && entry.has_error {
        ERROR_MARK.to_string()
    } else {
        String::new()
    };

    format!(
        r#"<span class="graph-block {:?}" style="width: {width}px; left: {left}px;">{title}{tooltip_container}</span>"#,
        entry.stage,
    )
}
