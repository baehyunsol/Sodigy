use crate::Span;

mod color;

pub use color::Color;

#[derive(Clone, Copy, Debug)]
pub struct RenderSpanOption {
    pub max_height: usize,
    pub max_width: usize,
    pub render_source: bool,
    pub color: Option<ColorOption>,
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

pub fn render_span(
    file_name: &str,
    bytes: &[u8],
    span: Span,
    extra_span: Option<Span>,
    option: RenderSpanOption,
) -> String {
    let extra_span = extra_span.unwrap_or(Span::None);
    let (primary_color, secondary_color) = match &option.color {
        Some(ColorOption { primary, secondary }) => (*primary, *secondary),
        None => (Color::None, Color::None),
    };

    if bytes.is_empty() || span == Span::None && extra_span == Span::None {
        todo!("nothing to render");
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

        for s in [span, extra_span] {
            match s {
                Span::Range { start, end, .. } => {
                    if i == start || i + 1 == end {
                        important_points.push((row, col));
                    }

                    if start <= i && i < end {
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
    let rendered_span = if width < option.max_width && height < option.max_height {
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
    }

    else {
        // Let's create a rect that includes important rects
        let important_rects: Vec<(usize, usize, usize, usize)> = important_points.into_iter().map(
            |(row, col)| (
                row.max(2) - 2,
                row + 2,
                col.max(5) - 5,
                col + 5,
            )
        ).collect();
        panic!("{:?}", (top, bottom, left, right))
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
