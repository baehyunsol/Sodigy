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
        let n = n.to_string();
        let pre = " ".repeat(8 - n.len());

        // do not color whitespaces!
        match &self.line_no {
            Color::None => format!("{pre}{n}"),
            Color::Blue => format!("{pre}{}", n.to_string().blue()),
            Color::Red => format!("{pre}{}", n.to_string().red()),
            Color::Yellow => format!("{pre}{}", n.to_string().yellow()),
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
        let mut non_ascii_chars = vec![];

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
                    if pos.is_none() {
                        pos = Some(line.get_pos());
                    }
                }

                if !line.non_ascii_chars.is_empty() && non_ascii_chars.len() < 8 {
                    for c in line.non_ascii_chars.iter() {
                        non_ascii_chars.push(*c);

                        if non_ascii_chars.len() == 8 {
                            break;
                        }
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
                "{}:{row}:{col}\n{}{}",
                file_session.render_file_hash(*file),
                result.join("\n"),
                alert_non_ascii_chars(&non_ascii_chars),
            )
        );
    }

    messages.join("\n\n")
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
                let dots = format!("       {}", colors.dots());
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
    line_no: usize,

    // ascii `c`, highlighted 'b'
    // if b { c + 128 } else { c }
    // non-ascii `x`, highlighted 'b'
    // if b { 128 } else { 0 }
    buffer: Vec<u8>,

    need_dots: bool,
    has_highlighted_char: bool,

    non_ascii_chars: Vec<u32>,
}

const MAX_LINE_LEN: usize = 88;

impl Line {
    pub fn new(line_no: usize, buffer: &[u8], non_ascii_chars: Vec<u8>) -> Self {
        let has_highlighted_char = buffer.iter().any(|c| *c >= 128);
        let mut need_dots = false;
        let mut buffer = if buffer.len() > MAX_LINE_LEN {
            need_dots = true;

            buffer[0..(MAX_LINE_LEN - 3)].to_vec()
        } else {
            buffer.to_vec()
        };

        let non_ascii_chars = if !non_ascii_chars.is_empty() {
            let s = String::from_utf8_lossy(&non_ascii_chars).to_string();

            s.chars().map(
                |c| c as u32
            ).collect()
        } else {
            vec![]
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
            line_no,
            buffer,
            has_highlighted_char,
            need_dots,
            non_ascii_chars,
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
            self.line_no + 1,
            self.buffer.iter().position(|c| *c >= 128).unwrap_or(0) + 1,
        )
    }

    pub fn render(&self, colors: ColorScheme) -> String {
        let bar = colors.bar();

        format!(
            "{} {bar} {}{}",
            colors.render_num(self.line_no + 1),  // human index starts with 1
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
    let mut non_ascii_chars = vec![];

    for (i, c) in content.iter().enumerate() {
        if *c == b'\n' {
            lines.push(Line::new(line_no, &curr_line, non_ascii_chars.clone()));
            curr_line.clear();
            line_no += 1;
            non_ascii_chars.clear();
            continue;
        }

        let mut mark = false;

        for (start, end) in spans.iter() {
            if *start <= i && i < *end {
                mark = true;
                break;
            }
        }

        // it doesn't render non-ascii-code chars
        if *c > 127 {
            curr_line.push(
                (mark as u8) * 128,
            );

            non_ascii_chars.push(*c);
        }

        else {
            curr_line.push(
                *c + (mark as u8) * 128,
            );
        }
    }

    if !curr_line.is_empty() {
        lines.push(Line::new(line_no, &curr_line, non_ascii_chars));
    }

    lines
}

fn remove_consecutive_underlines(lines: Vec<RenderedLine>) -> Vec<RenderedLine> {
    let mut buffer = Vec::with_capacity(lines.len());
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

                    buffer.extend(consec);
                    consec = vec![];
                }

                buffer.push(line);
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

        buffer.extend(consec);
    }

    buffer
}

fn push_underlines(lines: Vec<RenderedLine>) -> Vec<RenderedLine> {
    let mut buffer = Vec::with_capacity(lines.len() * 2);

    for line in lines.into_iter() {
        match line {
            RenderedLine::Normal(ref ln) => {
                let underlines = ln.get_underlines();
                buffer.push(line);

                if let Some(underlines) = underlines {
                    buffer.push(underlines);
                }
            },
            _ => {
                buffer.push(line);
            },
        }
    }

    buffer
}

fn alert_non_ascii_chars(non_ascii_chars: &[u32]) -> String {
    if non_ascii_chars.is_empty() {
        String::new()
    }

    else {
        let mut non_ascii_chars = non_ascii_chars.to_vec();
        non_ascii_chars.sort();
        non_ascii_chars.dedup();

        let (a, those, s) = if non_ascii_chars.len() == 1 {
            (" a", "the", "")
        } else {
            ("", "those", "s")
        };
        // unlucky that we cannot import `concat_commas` from sodigy_error
        let chars = match non_ascii_chars.len() {
            1 => format!("{}", render_u32_char(non_ascii_chars[0])),
            2 => format!("{} and {}", render_u32_char(non_ascii_chars[0]), render_u32_char(non_ascii_chars[1])),
            3 => format!(
                "{}, {} and {}",
                render_u32_char(non_ascii_chars[0]),
                render_u32_char(non_ascii_chars[1]),
                render_u32_char(non_ascii_chars[2]),
            ),
            _ => format!(
                "{}, {}, {}, ...",
                render_u32_char(non_ascii_chars[0]),
                render_u32_char(non_ascii_chars[1]),
                render_u32_char(non_ascii_chars[2]),
            ),
        };

        let 한글 = if non_ascii_chars.iter().any(|c| is_한글(c)) {
            String::from("\n코드에 한글이 있습니다. 한글을 사용하는 것은 전혀 문제가 없지만 오류 메시지에는 한글이 정상출력되지 않습니다. 오류 메시지에서 밑줄을 칠 때 각 글자의 크기를 계산해야하는데 한글의 크기는 계산하기 어렵거든요.")
        } else {
            String::new()
        };

        format!("\nNote: The code contains{a} non-ascii character{s} ({chars}). It's okay to use non-ascii characters, but the error messages cannot render {those} character{s} properly, due to the issues with font-size.{한글}")
    }
}

fn render_u32_char(c: u32) -> String {
    // it's guaranteed to be a valid utf-8
    format!("{:?}", char::from_u32(c).unwrap())
}

fn is_한글(c: &u32) -> bool {
    0x1100 <= *c && *c < 0x1200 || 0xac00 <= *c && *c < 0xd7a4
}
