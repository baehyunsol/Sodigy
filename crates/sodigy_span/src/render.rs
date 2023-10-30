use crate::SpanRange;
use colored::*;
use sodigy_files::global_file_session;
use std::collections::HashMap;

#[derive(Clone, Copy)]
enum Color {
    None,
    Red,
    Blue,
    Yellow,
}

#[derive(Clone, Copy)]
pub struct ColorScheme {
    line_no: Color,
    underline: Color,
}

impl ColorScheme {
    pub fn none() -> Self {
        ColorScheme {
            line_no: Color::None,
            underline: Color::None,
        }
    }

    pub fn error() -> Self {
        ColorScheme {
            line_no: Color::Blue,
            underline: Color::Red,
        }
    }

    pub fn warning() -> Self {
        ColorScheme {
            line_no: Color::Blue,
            underline: Color::Yellow,
        }
    }

    pub(crate) fn bar(&self) -> String {
        match &self.line_no {
            Color::None => String::from("│"),
            Color::Blue => format!("{}", "│".blue()),
            Color::Red => format!("{}", "│".red()),
            Color::Yellow => format!("{}", "│".yellow()),
        }
    }

    pub(crate) fn dots(&self) -> String {
        match &self.line_no {
            Color::None => String::from("..."),
            Color::Blue => format!("{}", "...".blue()),
            Color::Red => format!("{}", "...".red()),
            Color::Yellow => format!("{}", "...".yellow()),
        }
    }

    pub(crate) fn underline(&self) -> String {
        match &self.underline {
            Color::None => String::from("^"),
            Color::Blue => format!("{}", "^".blue()),
            Color::Red => format!("{}", "^".red()),
            Color::Yellow => format!("{}", "^".yellow()),
        }
    }

    pub(crate) fn l_arrow(&self) -> String {
        match &self.underline {
            Color::None => String::from(">"),
            Color::Blue => format!("{}", ">".blue()),
            Color::Red => format!("{}", ">".red()),
            Color::Yellow => format!("{}", ">".yellow()),
        }
    }

    pub(crate) fn render_num(&self, n: usize) -> String {
        let n = format!("{n}");
        let pre = " ".repeat(8 - n.len());

        // do not color whitespaces!
        match &self.line_no {
            Color::None => format!("{pre}{n}"),
            Color::Blue => format!("{pre}{}", format!("{n}").blue()),
            _ => todo!(),
        }
    }
}

// render spans for error messages
// (line numbers, underlines, path, ... etc)
pub fn render_spans(spans: &[SpanRange], color: ColorScheme) -> String {
    let mut spans_by_file: HashMap<u64, Vec<(usize, usize)>> = HashMap::new();

    for span in spans.iter() {
        match spans_by_file.get_mut(&span.file) {
            Some(v) => {
                v.push((span.start, span.end));
            },
            None => {
                spans_by_file.insert(span.file, vec![(span.start, span.end)]);
            },
        }
    }

    let file_session = unsafe { global_file_session() };
    let mut messages = Vec::with_capacity(spans_by_file.len());

    for (file, spans) in spans_by_file.iter() {
        let content = file_session.get_file_content(*file).unwrap();
        let lines = single_file(&content, spans);
        let lines_len = lines.len();
        let mut rendered_lines = Vec::with_capacity(lines_len * 2);
        let mut pos = None;

        for (i, line) in lines.iter().enumerate() {

            // this line doesn't have to be rendered
            if !line.has_highlighted_char
            && (i == 0 || !lines[i - 1].has_highlighted_char)
            && (i == lines_len - 1 || !lines[i + 1].has_highlighted_char) {

                // we don't need consecutive dots
                match rendered_lines.last() {
                    Some(RenderedLine::Dots) => {},
                    _ => {
                        rendered_lines.push(RenderedLine::Dots);
                    },
                }
            }

            else {
                if line.has_highlighted_char {
                    if let None = pos {
                        pos = Some(line.get_pos());
                    }
                }

                rendered_lines.push(RenderedLine::Normal(line.clone()));
            }
        }

        rendered_lines = remove_consecutive_underlines(rendered_lines);
        rendered_lines = push_underlines(rendered_lines);

        let mut result = Vec::with_capacity(rendered_lines.len());

        for line in rendered_lines.iter() {
            line.render(&mut result, color);
        }

        // remove leading whitespaces
        // TODO: O(n^2)
        while result.iter().all(
            |line| line.chars().next() == Some(' ')
        ) {
            result = result.iter().map(
                |line| line.get(1..).unwrap().to_string()
            ).collect();
        }

        let (row, col) = pos.unwrap();

        messages.push(
            format!(
                "{}:{row}:{col}\n{}",
                file_session.render_file_hash(*file),
                result.join("\n"),
            )
        );
    }

    format!(
        "{}", messages.join("\n\n")
    )
}

#[derive(Clone)]
enum RenderedLine {
    Normal(Line),
    Dots,
    Underline(Vec<bool>),
}

impl RenderedLine {
    pub fn render(&self, buffer: &mut Vec<String>, colors: ColorScheme) {
        let bar = colors.bar();
        let underline = colors.underline();
        let no_underline = String::from(" ");

        match self {
            RenderedLine::Normal(line) => {
                buffer.push(line.render(colors));
            },
            RenderedLine::Dots => {
                let dots = format!("         {bar} {}", colors.dots());
                let empty = format!("         {bar} ");

                buffer.push(empty.clone());
                buffer.push(dots);
                buffer.push(empty);
            },
            RenderedLine::Underline(mask) => {
                let underline = if mask.iter().all(|b| !*b) {
                    format!("{}{}", " ".repeat(MAX_LINE_LEN - 3), colors.l_arrow().repeat(3))
                } else {
                    mask.iter().map(
                        |b| if *b { underline.clone() } else { no_underline.clone() }
                    ).collect::<Vec<String>>().concat()
                };

                let line = format!(
                    "         {bar} {underline}",
                );

                buffer.push(line);
            },
        }
    }
}

#[derive(Clone)]
struct Line {
    index: usize,

    // ascii `c`, highlighted 'b'
    // if b { c + 128 } else { c }
    // non-ascii `x`, highlighted 'b'
    // if b { 128 } else { 0 }
    buffer: Vec<u8>,

    need_dots: bool,
    has_highlighted_char: bool,
}

const MAX_LINE_LEN: usize = 80;

impl Line {
    pub fn new(index: usize, buffer: &[u8]) -> Self {
        let has_highlighted_char = buffer.iter().any(|c| *c >= 128);
        let mut need_dots = false;
        let mut buffer = if buffer.len() > MAX_LINE_LEN {
            need_dots = true;

            buffer[0..(MAX_LINE_LEN - 3)].to_vec()
        } else {
            buffer.to_vec()
        };

        for c in buffer.iter_mut() {
            // don't underline indentations
            if *c == b' ' || *c == b' ' + 128 {
                *c = b' ';
            }

            else {
                break;
            }
        }

        Line {
            index,
            buffer,
            has_highlighted_char,
            need_dots,
        }
    }

    pub fn get_underlines(&self) -> Option<RenderedLine> {
        if self.has_highlighted_char {
            Some(RenderedLine::Underline(self.buffer.iter().map(|c| *c >= 128).collect()))
        }

        else {
            None
        }
    }

    pub fn get_pos(&self) -> (usize, usize) {
        (
            // human index starts with 1
            self.index + 1,
            self.buffer.iter().position(|c| *c >= 128).unwrap_or(0) + 1,
        )
    }

    pub fn render(&self, colors: ColorScheme) -> String {
        let bar = colors.bar();

        format!(
            "{} {bar} {}{}",
            colors.render_num(self.index + 1),  // human index starts with 1
            self.buffer.iter().map(
                |c| {
                    let c = if *c >= 128 {
                        (*c - 128) as char
                    } else {
                        *c as char
                    };

                    if c == '\0' {
                        '�'
                    } else {
                        c
                    }
                }
            ).collect::<String>(),
            if self.need_dots { colors.dots() } else { String::new() },
        )
    }
}

fn single_file(content: &[u8], spans: &Vec<(usize, usize)>) -> Vec<Line> {
    let mut lines = vec![];
    let mut curr_line = vec![];
    let mut line_no = 0;

    for (i, c) in content.iter().enumerate() {
        if *c == b'\n' {
            lines.push(Line::new(line_no, &curr_line));
            curr_line.clear();
            line_no += 1;
            continue;
        }

        let mut mark = false;

        for (start, end) in spans.iter() {
            if *start <= i && i < *end {
                mark = true;
                break;
            }
        }

        if *c > 127 {
            curr_line.push(
                (mark as u8) * 128,
            );
        }

        else {
            curr_line.push(
                *c + (mark as u8) * 128,
            );
        }
    }

    if !curr_line.is_empty() {
        lines.push(Line::new(line_no, &curr_line));
    }

    lines
}

fn remove_consecutive_underlines(lines: Vec<RenderedLine>) -> Vec<RenderedLine> {
    let mut buf = Vec::with_capacity(lines.len());
    let mut consec = vec![];

    for line in lines.into_iter() {
        match line {
            RenderedLine::Normal(ref ln) if ln.has_highlighted_char => {
                consec.push(line);
            },
            _ => {
                if !consec.is_empty() {
                    if consec.len() > 5 {
                        consec = vec![
                            consec[0..2].to_vec(),
                            vec![RenderedLine::Dots],
                            consec[(consec.len() - 2)..consec.len()].to_vec(),
                        ].concat();
                    }

                    buf.extend(consec);
                    consec = vec![];
                }

                buf.push(line);
            },
        }
    }

    if !consec.is_empty() {
        if consec.len() > 5 {
            consec = vec![
                consec[0..2].to_vec(),
                vec![RenderedLine::Dots],
                consec[(consec.len() - 2)..consec.len()].to_vec(),
            ].concat();
        }

        buf.extend(consec);
    }

    buf
}

fn push_underlines(lines: Vec<RenderedLine>) -> Vec<RenderedLine> {
    let mut buf = Vec::with_capacity(lines.len() * 2);

    for line in lines.into_iter() {
        match line {
            RenderedLine::Normal(ref ln) => {
                let underlines = ln.get_underlines();
                buf.push(line);

                if let Some(underlines) = underlines {
                    buf.push(underlines);
                }
            },
            _ => {
                buf.push(line);
            },
        }
    }

    buf
}
