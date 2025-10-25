use crate::Span;
use sodigy_file::File;
use std::collections::hash_map::{Entry, HashMap};

mod color;
mod session;

pub use color::Color;
pub use session::Session as RenderSpanSession;

#[derive(Clone, Debug)]
pub struct RenderableSpan {
    pub span: Span,
    pub auxiliary: bool,
    pub note: Option<String>,
}

#[derive(Clone, Debug)]
pub struct RenderSpanOption {
    pub max_height: usize,
    pub max_width: usize,
    pub render_source: bool,
    pub color: Option<ColorOption>,

    // It it's none, it uses 2 newline characters.
    pub group_delim: Option<(String, Color)>,
}

#[derive(Clone, Copy, Debug)]
pub struct ColorOption {
    pub primary: Color,
    pub auxiliary: Color,
    pub info: Color,
}

struct Line {
    content: Vec<u8>,
    colors: Vec<Option<Color>>,  // of underline
    notes: Vec<(usize, String)>,  // (index, note)
}

// NOTE: In rust, when an identifier is very very long,
//     1. span is rendered like "1 | ...et abc...def = ..."
//        - it truncates prefix, postfix, and even the mid of the span and of course underlines
//        - it shows 3 characters before the first character of the span and 3 characters after the last character of the span!
//     2. the identifier in the error message is not truncated (hence the terminal is laggy)

// Some spans are close to each other, some are far, some are even in different files.
// It first decides which spans to group together, render each groups, and joins them.
// It returns an empty string if there's nothing to render.
pub fn render_spans(
    spans: &[RenderableSpan],
    option: &RenderSpanOption,
    session: &mut RenderSpanSession,
) -> String {
    if spans.is_empty() {
        return String::new();
    }

    let mut spans_by_file: HashMap<File, Vec<(RenderableSpan, (usize, usize, usize, usize))>> = HashMap::new();
    let mut files_with_empty_rects = HashMap::new();

    for span in spans.iter() {
        let Some(file) = span.span.get_file() else { continue };
        let Some(rect) = session.get_rect(span.span) else {
            files_with_empty_rects.insert(file, span.clone());
            continue;
        };

        match spans_by_file.entry(file) {
            Entry::Occupied(mut e) => {
                e.get_mut().push((span.clone(), rect));
            },
            Entry::Vacant(e) => {
                e.insert(vec![(span.clone(), rect)]);
            },
        }
    }

    // TODO: sort groups
    //       1. a group with non-auxiliary spans has to come before auxiliary-only group
    //       2. if there are ties, they're ordered by y-position
    let mut groups = Vec::with_capacity(spans_by_file.len());

    for span in files_with_empty_rects.values() {
        groups.push(render_close_spans(&[span.clone()], option, session));
    }

    for (file, spans) in spans_by_file.into_iter() {
        match spans.len() {
            0 => {},
            1 => {
                groups.push(render_close_spans(&[spans[0].0.clone()], option, session));
            },
            2 => {
                let merged_rect = merge_rects(spans[0].1, spans[1].1);
                let (merged_w, merged_h) = (merged_rect.2 - merged_rect.0, merged_rect.3 - merged_rect.1);

                if merged_w > option.max_width || merged_h > option.max_height {
                    groups.push(render_close_spans(&[spans[0].0.clone()], option, session));
                    groups.push(render_close_spans(&[spans[1].0.clone()], option, session));
                }

                else {
                    groups.push(render_close_spans(&[spans[0].0.clone(), spans[1].0.clone()], option, session));
                }
            },
            // TODO: I have to group close spans... but I'm too lazy to do that
            3.. => todo!(),
        }
    }

    groups = groups.into_iter().filter(|g| !g.is_empty()).collect();
    let delim = option.group_delim.as_ref().map(|(delim, color)| color.render_fg(delim)).unwrap_or_else(|| String::from("\n\n"));
    groups.join(&delim)
}

// It assumes that all the spans are close together, and tries to render all spans in a single window.
// It returns an empty string if there's nothing to render.
fn render_close_spans(
    spans: &[RenderableSpan],
    option: &RenderSpanOption,
    session: &mut RenderSpanSession,
) -> String {
    let (primary_color, auxiliary_color, info_color) = match &option.color {
        Some(ColorOption { primary, auxiliary, info }) => (*primary, *auxiliary, *info),
        None => (Color::None, Color::None, Color::None),
    };

    let file_name = session.get_path(spans[0].span);
    let bytes = session.get_bytes(spans[0].span);

    if let (Some(file_name), None) = (&file_name, &bytes) {
        return format!(
            "{}: {file_name}",
            info_color.render_fg("src"),
        );
    }

    else if let (None, None) = (&file_name, &bytes) {
        return String::new();
    }

    let file_name = file_name.unwrap();
    let bytes = bytes.unwrap();

    if bytes.is_empty() || spans.iter().all(|span| span.span.get_file().is_none()) {
        return String::new();
    }

    let mut row = 0;
    let mut col = 0;
    let mut beginning_row_col = None;

    let mut lines = vec![];
    let mut curr_bytes = vec![];
    let mut curr_colors = vec![];
    let mut curr_notes = vec![];

    // In order to underline every character in the span, this rect has to be rendered.
    // Coordinates are inclusive!
    let (mut top, mut bottom, mut left, mut right) = (usize::MAX, 0, usize::MAX, 0);

    for (i, b) in bytes.iter().enumerate() {
        let mut curr_byte_color = None;

        for s in spans.iter() {
            match s.span {
                Span::Range { start, end, .. } => {
                    if i == start && s.note.is_some() {
                        curr_notes.push((col, s.note.clone().unwrap()));
                    }

                    if start <= i && i < end {
                        top = top.min(row);
                        bottom = bottom.max(row);
                        left = left.min(col);
                        right = right.max(col);
                        curr_byte_color = Some(primary_color);

                        if beginning_row_col.is_none() {
                            beginning_row_col = Some((row + 1, col + 1));
                        }
                    }
                },
                Span::None => {},
                _ => panic!("TODO: {s:?}"),
            }
        }

        match b {
            b'\n' => {
                lines.push(Line {
                    content: curr_bytes,
                    colors: curr_colors,
                    notes: curr_notes,
                });
                curr_bytes = vec![];
                curr_colors = vec![];
                curr_notes = vec![];
                row += 1;
                col = 0;
            },
            b'\r' | b'\t' => {  // not pretty when rendered
                curr_bytes.push(b' ');
                curr_colors.push(curr_byte_color);
                col += 1;
            },
            _ => {
                curr_bytes.push(*b);
                curr_colors.push(curr_byte_color);
                col += 1;
            },
        }
    }

    if !curr_bytes.is_empty() {
        lines.push(Line {
            content: curr_bytes,
            colors: curr_colors,
            notes: curr_notes,
        });
    }

    let rendered_span = {
        let (top, bottom) = match (top, bottom) {
            (0, _) => (0, 2),
            (1, _) => (0, 3),
            _ => (top - 2, bottom + 2),
        };
        let (left, right) = match (left, right) {
            (_, r) if r < option.max_width - 20 => (0, option.max_width - 10),
            _ => (right + 20 - option.max_width, right + 10),
        };

        render_span_worker(
            lines,
            (top, bottom, left, right),
            primary_color,
            auxiliary_color,
            info_color,
        )
    };

    let rendered_source = if option.render_source {
        format!(
            "{}: {file_name}{}\n",
            info_color.render_fg("src"),
            if let Some((row, col)) = beginning_row_col {
                format!(":{row}:{col}")
            } else {
                String::new()
            },
        )
    } else {
        String::new()
    };

    format!("{rendered_source}{rendered_span}")
}

// It renders `lines`, but only in `rect`.
fn render_span_worker(
    lines: Vec<Line>,
    // all numbers are inclusive
    rect: (usize, usize, usize, usize),
    primary_color: Color,
    auxiliary_color: Color,
    info_color: Color,
) -> String {
    let mut result = vec![];
    let (top, bottom, left, right) = rect;

    for (row, line) in lines.iter().enumerate() {
        if row < top {
            continue;
        }

        if row > bottom {
            break;
        }

        let line_no = if bottom >= 999 {
            format!("{:>6}", row + 1)
        } else {
            format!("{:>4}", row + 1)
        };
        let line_bar = " | ";
        let pre_dots = if left > 0 { " ... " } else { "" };

        result.push(format!(
            "{}{}{}{}{}",
            info_color.render_fg(&line_no),
            info_color.render_fg(&line_bar),
            auxiliary_color.render_fg(&pre_dots),
            String::from_utf8_lossy(&line.content[left.min(line.content.len())..(right + 1).min(line.content.len())]),
            if right < line.content.len() {
                auxiliary_color.render_fg(" ... ")
            } else {
                String::new()
            },
        ));

        if line.colors.iter().any(|c| c.is_some()) {
            result.push(format!(
                "{}{}{}{}",
                " ".repeat(line_no.len()),
                info_color.render_fg(&line_bar),
                " ".repeat(pre_dots.len()),
                line.colors[left.min(line.colors.len())..(right + 1).min(line.colors.len())].iter().map(
                    |color| match color {
                        Some(c) => c.render_fg("^"),
                        None => String::from(" "),
                    }
                ).collect::<Vec<_>>().concat(),
            ));
        }

        // TODO: add labels and write notes underneath
        //
        // 1. when the labels are far from each other
        //         ^^^        ^^^
        //         |          |
        //        (0)        (1)
        //
        // 2. when the labels are far from each other, but one of index is 0
        //    ^^^^
        //    |
        //    *--(0) 
        //
        // 3. when the labels are close to each other
        //         ^ ^
        //         | |
        //         | *--(1)
        //         |
        //         *--(0)
        //
        if !line.notes.is_empty() {
            let mut labels = vec![];
            let mut max_depth = 1;
            struct Label {
                pub depth: usize,
                pub x: usize,
                pub asterisk: bool,
            }

            // notes are always sorted by x
            for (i, (x, _)) in line.notes.iter().enumerate() {
                let x = *x;

                match labels.last_mut() {
                    Some(Label { x: last_x, .. }) if *last_x + 6 < x => {
                        labels.push(Label { depth: 1, x, asterisk: false });
                    },
                    // TODO: I know it'll be broken if more than 3 spans are very close to each other... but I'm too lazy to fix it.
                    Some(Label { depth, asterisk, .. }) => {
                        *depth += 1;
                        *asterisk = true;
                        max_depth = max_depth.max(*depth);
                        labels.push(Label { depth: 1, x, asterisk: true });
                    },
                    None => {
                        labels.push(Label { depth: 1, x, asterisk: x == 0 });
                    },
                }
            }

            let mut label_lines = vec![vec![b' '; line.content.len() + 7]; max_depth * 2];

            for (i, label) in labels.iter().enumerate() {
                for y in 0..(label.depth * 2 - 1) {
                    label_lines[y][label.x] = b'|';
                }

                if label.asterisk {
                    label_lines[label.depth * 2 - 1][label.x] = b'*';
                    label_lines[label.depth * 2 - 1][label.x + 1] = b'-';
                    label_lines[label.depth * 2 - 1][label.x + 2] = b'-';
                    label_lines[label.depth * 2 - 1][label.x + 3] = b'(';

                    // I guess there's not gonna be more than 100 labels... right?
                    if i >= 10 {
                        label_lines[label.depth * 2 - 1][label.x + 4] = (i as u8 / 10) + b'0';
                        label_lines[label.depth * 2 - 1][label.x + 5] = (i as u8 % 10) + b'0';
                        label_lines[label.depth * 2 - 1][label.x + 6] = b')';
                    }

                    else {
                        label_lines[label.depth * 2 - 1][label.x + 4] = i as u8 + b'0';
                        label_lines[label.depth * 2 - 1][label.x + 5] = b')';
                    }
                }

                else {
                    label_lines[label.depth * 2 - 1][label.x - 1] = b'(';

                    // I guess there's not gonna be more than 100 labels... right?
                    if i >= 10 {
                        label_lines[label.depth * 2 - 1][label.x] = (i as u8 / 10) + b'0';
                        label_lines[label.depth * 2 - 1][label.x + 1] = (i as u8 % 10) + b'0';
                        label_lines[label.depth * 2 - 1][label.x + 2] = b')';
                    }

                    else {
                        label_lines[label.depth * 2 - 1][label.x] = i as u8 + b'0';
                        label_lines[label.depth * 2 - 1][label.x + 1] = b')';
                    }
                }
            }

            // an empty line for readability
            label_lines.push(vec![]);

            for (i, (_, note)) in line.notes.iter().enumerate() {
                // TODO: automatically break lines if a note is too long
                label_lines.push(format!("({i}): {note}").into_bytes());
            }

            for line in label_lines.iter() {
                result.push(format!(
                    "{}{}{}{}",
                    " ".repeat(line_no.len()),
                    info_color.render_fg(&line_bar),
                    " ".repeat(pre_dots.len()),
                    auxiliary_color.render_fg(&String::from_utf8_lossy(line)),
                ));
            }
        }
    }

    result.join("\n")
}

fn merge_rects(r1: (usize, usize, usize, usize), r2: (usize, usize, usize, usize)) -> (usize, usize, usize, usize) {
    (
        r1.0.min(r2.0),
        r1.1.min(r2.1),
        r1.2.max(r2.2),
        r1.3.max(r2.3),
    )
}
