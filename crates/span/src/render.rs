use crate::Span;
use sodigy_file::File;
use std::collections::hash_map::{Entry, HashMap};

mod color;
mod session;

pub use color::Color;
pub use session::Session as RenderSpanSession;

#[derive(Clone, Debug, Eq, Hash, PartialEq)]
pub struct RenderableSpan {
    pub span: Span,
    pub auxiliary: bool,
    pub note: Option<String>,
}

#[derive(Clone, Debug)]
pub struct RenderSpanOption {
    pub max_height: usize,
    pub max_width: usize,
    pub context: usize,
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

    // Spans in a "group" are close to each other.
    // The renderer will render a group at a time.
    let mut groups: Vec<Vec<RenderableSpan>> = Vec::with_capacity(spans_by_file.len());

    for span in files_with_empty_rects.values() {
        groups.push(vec![span.clone()]);
    }

    for (file, spans) in spans_by_file.into_iter() {
        match spans.len() {
            0 => {},
            1 => {
                groups.push(vec![spans[0].0.clone()]);
            },
            2.. => {
                let mut groups_in_this_file: Vec<(Vec<RenderableSpan>, (usize, usize, usize, usize))> = vec![];

                for (span, rect) in spans.iter() {
                    let mut found_groupable_span = false;

                    for (spans, group_rect) in groups_in_this_file.iter_mut() {
                        let merged_rect = merge_rects(*rect, *group_rect);
                        let (merged_w, merged_h) = (merged_rect.2 - merged_rect.0, merged_rect.3 - merged_rect.1);

                        // Merging this span to this group doesn't make the rect bigger,
                        // which means the group or the span is subset of the other.
                        // So, we can group them together.
                        let condition1 = merged_rect == *group_rect || merged_rect == *rect;

                        // Merged rect is small enough, so we can group them together.
                        let condition2 = merged_w <= option.max_width && merged_h <= option.max_height;

                        if condition1 || condition2 {
                            spans.push(span.clone());
                            *group_rect = merged_rect;
                            found_groupable_span = true;
                            break;
                        }
                    }

                    if !found_groupable_span {
                        groups_in_this_file.push((vec![span.clone()], *rect));
                    }
                }

                for (spans, _) in groups_in_this_file.into_iter() {
                    groups.push(spans);
                }
            },
        }
    }

    // 1. auxiliary spans come after important ones
    // 2. other than that, groups are sorted by the first span
    groups.sort_by_key(|spans| spans[0].span);
    groups.sort_by_key(|spans| !spans.iter().any(|span| !span.auxiliary) as u8);

    let mut rendered_groups = Vec::with_capacity(groups.len());
    let mut label_index_offset = 0;

    for spans in groups.iter() {
        rendered_groups.push(render_close_spans(
            spans,
            option,
            session,
            label_index_offset,
        ));

        label_index_offset += spans.iter().map(|span| if span.note.is_some() { 1 } else { 0 }).sum::<usize>();
    }

    rendered_groups = rendered_groups.into_iter().filter(|g| !g.is_empty()).collect();
    let delim = option.group_delim.as_ref().map(|(delim, color)| color.render_fg(delim)).unwrap_or_else(|| String::from("\n\n"));
    rendered_groups.join(&delim)
}

// It assumes that all the spans are close together, and tries to render all spans in a single window.
// It returns an empty string if there's nothing to render.
fn render_close_spans(
    spans: &[RenderableSpan],
    option: &RenderSpanOption,
    session: &mut RenderSpanSession,
    label_index_offset: usize,
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
                        curr_byte_color = Some(if s.auxiliary { auxiliary_color } else { primary_color });

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
        let (top, bottom) = (top.max(option.context) - option.context, bottom + option.context);
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
            option.max_width,
            label_index_offset,
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

enum RenderedLineKind {
    Code,
    Underline,
    LabelMarker,
    Note,
    Trunc,
}

// It renders `lines`, but only in `rect`.
fn render_span_worker(
    lines: Vec<Line>,
    // all numbers are inclusive
    rect: (usize, usize, usize, usize),
    primary_color: Color,
    auxiliary_color: Color,
    info_color: Color,
    max_width: usize,
    label_index_offset: usize,
) -> String {
    let mut code_lines: Vec<(String, RenderedLineKind)> = vec![];
    let mut note_lines: Vec<(String, RenderedLineKind)> = vec![];
    let (top, bottom, left, right) = rect;
    let mut label_index = label_index_offset;

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

        code_lines.push((
            format!(
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
            ),
            RenderedLineKind::Code,
        ));

        if line.colors.iter().any(|c| c.is_some()) {
            code_lines.push((
                format!(
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
                ),
                RenderedLineKind::Underline,
            ));
        }

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

            // Do you see the parenthesis with numbers in it? We call them `LabelMarker`.
            // `x` is the position of the marker (absolute).
            // A marker may have multiple `index`es, so it's `Vec<usize>`.
            #[derive(Clone, Debug)]
            struct LabelMarker {
                pub depth: usize,
                pub x: usize,
                pub asterisk: bool,
                pub index: Vec<usize>,
                pub far_left: bool,
            }

            // notes are always sorted by x
            for (i, (x, note)) in line.notes.iter().enumerate() {
                fn make_labels_deeper(
                    labels: &mut Vec<LabelMarker>,
                    mut curr_x: usize,
                    mut curr_depth: usize,
                    new_index: usize,
                ) -> (usize, bool) {  // (depth, has_same_index)
                    for LabelMarker { x, depth, asterisk, index, far_left: _ } in labels.iter_mut().rev() {
                        if *x == curr_x {
                            index.push(new_index);
                            return (curr_depth, true);
                        }

                        else if *x + 6 >= curr_x {
                            *depth += 1;
                            *asterisk = true;
                            curr_x = *x;
                            curr_depth = *depth;
                        }

                        else {
                            return (curr_depth, false);
                        }
                    }

                    (curr_depth, false)
                }

                let mut x = *x;
                let far_left = if x < left { x = left - 1; true } else { false };
                let (new_depth, has_same_index) = make_labels_deeper(&mut labels, x, 1, label_index + i);
                max_depth = new_depth.max(max_depth);

                if !has_same_index {
                    labels.push(LabelMarker { depth: 1, x, asterisk: x == 0, index: vec![label_index + i], far_left });
                }

                let note_no = format!("({}): ", label_index + i);
                let line_max_width = (max_width.max(note_no.len()) - note_no.len()).max(20);

                for (j, note_line) in break_lines(note, line_max_width).iter().enumerate() {
                    note_lines.push((
                        auxiliary_color.render_fg(&format!(
                            "{}{note_line}",
                            if j == 0 {
                                note_no.clone()
                            } else {
                                " ".repeat(note_no.len())
                            },
                        )),
                        RenderedLineKind::Note,
                    ));
                }
            }

            let mut label_lines = vec![vec![b' '; line.content.len() + 7]; max_depth * 2];

            for label in labels.iter() {
                let index_rendered = format!("({})", label.index.iter().map(|i| i.to_string()).collect::<Vec<_>>().join(",")).into_bytes();

                if label.far_left {
                    label_lines[label.depth * 2 - 1][0] = b'<';
                    label_lines[label.depth * 2 - 1][1] = b'-';

                    for (i, c) in index_rendered.iter().enumerate() {
                        label_lines[label.depth * 2 - 1][i + 2] = *c;
                    }
                }

                else {
                    let x = label.x - left;

                    for y in 0..(label.depth * 2 - 1) {
                        label_lines[y][x] = b'|';
                    }

                    if label.asterisk {
                        label_lines[label.depth * 2 - 1][x] = b'*';
                        label_lines[label.depth * 2 - 1][x + 1] = b'-';
                        label_lines[label.depth * 2 - 1][x + 2] = b'-';

                        for (i, c) in index_rendered.iter().enumerate() {
                            label_lines[label.depth * 2 - 1][x + 3 + i] = *c;
                        }
                    }

                    else {
                        let offset = index_rendered.len() / 2;

                        for (i, c) in index_rendered.iter().enumerate() {
                            label_lines[label.depth * 2 - 1][x + i - offset] = *c;
                        }
                    }
                }
            }

            label_index += labels.len();

            for mut line in label_lines.into_iter() {
                while let Some(b' ') = line.last() {
                    line.pop().unwrap();
                }

                code_lines.push((
                    format!(
                        "{}{}{}{}",
                        " ".repeat(line_no.len()),
                        info_color.render_fg(&line_bar),
                        " ".repeat(pre_dots.len()),
                        auxiliary_color.render_fg(&String::from_utf8_lossy(&line)),
                    ),
                    RenderedLineKind::LabelMarker,
                ));
            }
        }
    }

    if !note_lines.is_empty() {
        code_lines.push((String::new(), RenderedLineKind::Note));  // an empty line for readability
        code_lines.extend(note_lines);
    }

    code_lines = cut_long_underlines(code_lines);

    let code_lines = code_lines.into_iter().map(|(content, _)| content).collect::<Vec<_>>();
    code_lines.join("\n")
}

fn merge_rects(r1: (usize, usize, usize, usize), r2: (usize, usize, usize, usize)) -> (usize, usize, usize, usize) {
    (
        r1.0.min(r2.0),
        r1.1.min(r2.1),
        r1.2.max(r2.2),
        r1.3.max(r2.3),
    )
}

fn break_lines(s: &str, max_width: usize) -> Vec<String> {
    let mut curr_line = vec![];
    let mut lines = vec![];
    let long_enough = (max_width.max(8) - 8).max(max_width * 4 / 5);

    // It assumes that every character has the same width.
    // It assumes that there's no newline character.
    for ch in s.chars() {
        if curr_line.len() >= max_width {
            lines.push(curr_line);
            curr_line = vec![];

            if ch != ' ' {
                curr_line.push(ch);
            }
        }

        else if curr_line.len() >= long_enough && ch == ' ' {
            lines.push(curr_line);
            curr_line = vec![];
        }

        else {
            curr_line.push(ch);
        }
    }

    if !curr_line.is_empty() {
        lines.push(curr_line);
    }

    lines.into_iter().map(|chs| chs.into_iter().collect()).collect()
}

// If there are more than 5 consecutive lines that are 1) all underlined and 2) have no label markers at all,
// it leaves the first 2 and the last 2 lines and truncate the other lines.
fn cut_long_underlines(lines: Vec<(String, RenderedLineKind)>) -> Vec<(String, RenderedLineKind)> {
    if lines.len() <= 5 {
        lines
    }

    else {
        let mut cursor = 0;
        let mut consecutive_underlines = 0;
        let mut lines_to_erase = vec![];

        loop {
            match (lines.get(cursor), lines.get(cursor + 1)) {
                (Some((_, RenderedLineKind::Code)), Some((_, RenderedLineKind::Underline))) => {
                    cursor += 2;
                    consecutive_underlines += 1;
                },
                _ => {
                    if consecutive_underlines > 5 {
                        // first 2 and the last 2 lines survive. the other lines all die
                        for i in (cursor - consecutive_underlines * 2 + 4)..(cursor - 4) {
                            lines_to_erase.push(i);
                        }
                    }

                    cursor += 1;
                    consecutive_underlines = 0;

                    if cursor >= lines.len() {
                        break;
                    }
                },
            }
        }

        if lines_to_erase.is_empty() {
            lines
        }

        else {
            let mut result = Vec::with_capacity(lines.len() - lines_to_erase.len());
            let mut has_to_insert_dotdotdot = true;

            for (i, line) in lines.into_iter().enumerate() {
                // It's O(n^2), but n is small!!
                if lines_to_erase.contains(&i) {
                    if has_to_insert_dotdotdot {
                        // TODO: color this
                        // TODO: smarter way to indent the dots?
                        result.push((String::new(), RenderedLineKind::Trunc));
                        result.push((String::from("    ..."), RenderedLineKind::Trunc));
                        result.push((String::new(), RenderedLineKind::Trunc));
                        has_to_insert_dotdotdot = false;
                    }

                    continue;
                }

                else {
                    result.push(line);
                    has_to_insert_dotdotdot = true;
                }
            }

            result
        }
    }
}
