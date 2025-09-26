use sodigy_span::Span;

#[derive(Clone, Copy, Debug)]
pub struct RenderSpanOption {
    pub max_height: usize,
    pub max_width: usize,
    pub color: bool,
}

struct Line {
    content: Vec<u8>,
    underline_byte: Vec<bool>,
    underline_line: bool,    // self.underline_byte.any(|b| b)
}

// NOTE: In rust, when an identifier is very very long,
//     1. span is rendered like "1 | ...et abc...def = ..."
//        - it truncates prefix, postfix, and even the mid of the span and of course underlines
//        - it shows 3 characters before the first character of the span and 3 characters after the last character of the span!
//     2. the identifier in the error message is not truncated (hence the terminal is laggy)

// TODO: It's just PoC.
pub fn render_span(bytes: &[u8], span: Span, extra_span: Option<Span>, option: RenderSpanOption) -> String {
    let extra_span = extra_span.unwrap_or(Span::None);

    if bytes.is_empty() || span == Span::None && extra_span == Span::None {
        todo!("nothing to render");
    }

    let mut row = 0;
    let mut col = 0;

    let mut lines = vec![];
    let mut curr_line_bytes = vec![];
    let mut curr_line_underlines = vec![];
    let mut curr_line_underline = false;

    // Even if a span is very long, it'll render the first and the last character of the span.
    let mut important_points: Vec<(usize, usize)> = vec![];

    // In order to underline every character in the span, this rect has to be rendered.
    // Coordinates are inclusive!
    let (mut top, mut bottom, mut left, mut right) = (usize::MAX, 0, usize::MAX, 0);

    for (i, b) in bytes.iter().enumerate() {
        let mut curr_byte_underline = false;

        for s in [span, extra_span] {
            match s {
                Span::Range { start, end, .. } => {
                    if i == start && i + 1 == end {
                        important_points.push((row, col));
                    }

                    if start <= i && i < end {
                        top = top.min(row);
                        bottom = bottom.max(row);
                        left = left.min(col);
                        right = right.max(col);
                        curr_byte_underline = true;
                        curr_line_underline = true;
                    }
                },
                Span::None => {},
                _ => panic!("TODO: {s:?}"),
            }
        }

        match b {
            b'\n' => {
                lines.push(Line {
                    content: curr_line_bytes,
                    underline_byte: curr_line_underlines,
                    underline_line: curr_line_underline,
                });
                curr_line_bytes = vec![];
                curr_line_underlines = vec![];
                curr_line_underline = false;
                row += 1;
                col = 0;
            },
            b'\r' | b'\t' => {  // not pretty when rendered
                curr_line_bytes.push(b' ');
                curr_line_underlines.push(curr_byte_underline);
                col += 1;
            },
            _ => {
                curr_line_bytes.push(*b);
                curr_line_underlines.push(curr_byte_underline);
                col += 1;
            },
        }
    }

    if !curr_line_bytes.is_empty() {
        lines.push(Line {
            content: curr_line_bytes,
            underline_byte: curr_line_underlines,
            underline_line: curr_line_underline,
        });
    }

    let (width, height) = (right - left + 1, bottom - top + 1);

    if width < option.max_width && height < option.max_height {
        return render_span_worker(
            lines,
            // TODO: smarter contexts...
            // TODO: when comparing `width`, `height` and `max_width`, `max_height`, it has to count the context (padding)
            (
                top.max(2) - 2,
                bottom + 2,
                left.max(25) - 25,
                right + 25,
            ),
        );
    }

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
}

fn render_span_worker(
    lines: Vec<Line>,
    rect: (usize, usize, usize, usize),
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
        let pre_dots = if left > 0 {
            " ... "
        } else {
            ""
        };

        result.push(format!(
            "{line_no}{line_bar}{pre_dots}{}{}",
            String::from_utf8_lossy(&line.content[left.min(line.content.len())..(right + 1).min(line.content.len())]),
            if right < line.content.len() { " ... " } else { "" },
        ));

        if line.underline_line {
            result.push(format!(
                "{}{line_bar}{}{}",
                " ".repeat(line_no.len()),
                " ".repeat(pre_dots.len()),
                line.underline_byte[left.min(line.underline_byte.len())..(right + 1).min(line.underline_byte.len())].iter().map(
                    |underline| if *underline { "^" } else { " " }
                ).collect::<Vec<_>>().concat(),
            ));
        }
    }

    result.join("\n")
}
