use crate::Span;
use sodigy_file::File;
use std::collections::hash_map::{Entry, HashMap};

mod color;
mod session;

pub use color::Color;
pub use session::Session as RenderSpanSession;

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
    pub secondary: Color,
}

struct Line {
    content: Vec<u8>,
    colors: Vec<Option<Color>>,  // of underline
    is_colored: bool,  // `colors.any(|c| c.is_some())`
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
    spans: Vec<Span>,
    option: &RenderSpanOption,
    session: &mut RenderSpanSession,
) -> String {
    if spans.is_empty() {
        return String::new();
    }

    let mut spans_by_file: HashMap<File, Vec<(Span, (usize, usize, usize, usize))>> = HashMap::new();
    let mut files_with_empty_rects = HashMap::new();

    for span in spans.iter() {
        let Some(file) = span.get_file() else { continue };
        let Some(rect) = session.get_rect(*span) else {
            files_with_empty_rects.insert(file, *span);
            continue;
        };

        match spans_by_file.entry(file) {
            Entry::Occupied(mut e) => {
                e.get_mut().push((*span, rect));
            },
            Entry::Vacant(e) => {
                e.insert(vec![(*span, rect)]);
            },
        }
    }

    // TODO: sort groups
    //       1. by importance
    //       2. by file name and span
    let mut groups = Vec::with_capacity(spans_by_file.len());

    for span in files_with_empty_rects.values() {
        groups.push(render_close_spans(vec![*span], option, session));
    }

    for (file, spans) in spans_by_file.into_iter() {
        match spans.len() {
            0 => {},
            1 => {
                groups.push(render_close_spans(vec![spans[0].0], option, session));
            },
            // TODO: I have to group close spans... but I'm too lazy to do that
            2.. => todo!(),
        }
    }

    groups = groups.into_iter().filter(|g| !g.is_empty()).collect();
    let delim = option.group_delim.as_ref().map(|(delim, color)| color.render_fg(delim)).unwrap_or_else(|| String::from("\n\n"));
    groups.join(&delim)
}

// It assumes that all the spans are close together, and tries to render all spans in a single window.
// It returns an empty string if there's nothing to render.
fn render_close_spans(
    spans: Vec<Span>,
    option: &RenderSpanOption,
    session: &mut RenderSpanSession,
) -> String {
    let (primary_color, secondary_color) = match &option.color {
        Some(ColorOption { primary, secondary }) => (*primary, *secondary),
        None => (Color::None, Color::None),
    };

    let file_name = session.get_path(spans[0]);
    let bytes = session.get_bytes(spans[0]);

    if let (Some(file_name), None) = (&file_name, &bytes) {
        return format!(
            "{}: {file_name}",
            secondary_color.render_fg("src"),
        );
    }

    else if let (None, None) = (&file_name, &bytes) {
        return String::new();
    }

    let file_name = file_name.unwrap();
    let bytes = bytes.unwrap();

    if bytes.is_empty() || spans.iter().all(|span| span.get_file().is_none()) {
        return String::new();
    }

    let mut row = 0;
    let mut col = 0;
    let mut beginning_row_col = None;

    let mut lines = vec![];
    let mut curr_bytes = vec![];
    let mut curr_colors = vec![];
    let mut is_colored = false;

    // Even if a span is very long, it'll render the first and the last character of the span.
    let mut important_points: Vec<(usize, usize)> = vec![];

    // In order to underline every character in the span, this rect has to be rendered.
    // Coordinates are inclusive!
    let (mut top, mut bottom, mut left, mut right) = (usize::MAX, 0, usize::MAX, 0);

    for (i, b) in bytes.iter().enumerate() {
        let mut curr_byte_color = None;

        for s in spans.iter() {
            match s {
                Span::Range { start, end, .. } => {
                    if i == *start || i + 1 == *end {
                        important_points.push((row, col));
                    }

                    if *start <= i && i < *end {
                        top = top.min(row);
                        bottom = bottom.max(row);
                        left = left.min(col);
                        right = right.max(col);
                        curr_byte_color = Some(primary_color);
                        is_colored = true;

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
                    is_colored,
                });
                curr_bytes = vec![];
                curr_colors = vec![];
                is_colored = false;
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
            is_colored,
        });
    }

    let (width, height) = (right - left + 1, bottom - top + 1);

    // TODO: when comparing `width`, `height` and `max_width`, `max_height`, it has to count the context (padding)
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
            secondary_color,
        )
    };

    let rendered_source = if option.render_source {
        format!(
            "{}: {file_name}{}\n",
            secondary_color.render_fg("src"),
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
    secondary_color: Color,
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
            secondary_color.render_fg(&line_no),
            secondary_color.render_fg(&line_bar),
            secondary_color.render_fg(&pre_dots),
            String::from_utf8_lossy(&line.content[left.min(line.content.len())..(right + 1).min(line.content.len())]),
            if right < line.content.len() {
                secondary_color.render_fg(" ... ")
            } else {
                String::new()
            },
        ));

        if line.is_colored {
            result.push(format!(
                "{}{}{}{}",
                " ".repeat(line_no.len()),
                secondary_color.render_fg(&line_bar),
                secondary_color.render_fg(&" ".repeat(pre_dots.len())),
                line.colors[left.min(line.colors.len())..(right + 1).min(line.colors.len())].iter().map(
                    |color| match color {
                        Some(c) => c.render_fg("^"),
                        None => String::from(" "),
                    }
                ).collect::<Vec<_>>().concat(),
            ));
        }
    }

    result.join("\n")
}
