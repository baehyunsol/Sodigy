use super::remove_ansi_characters;
use std::fmt;

#[derive(Clone, Debug)]
pub enum LineMatcher {
    AnyLines,
    Tokens(Vec<Token>),
}

impl fmt::Display for LineMatcher {
    fn fmt(&self, fmt: &mut fmt::Formatter) -> Result<(), fmt::Error> {
        let s = match self {
            LineMatcher::AnyLines => String::from("......"),
            LineMatcher::Tokens(tokens) => tokens.iter().map(
                |token| match token {
                    Token::Skip => String::from("..."),
                    Token::Text(s) => String::from_utf8_lossy(s).to_string(),
                }
            ).collect::<Vec<_>>().concat(),
        };

        write!(fmt, "{s}")
    }
}

#[derive(Clone, Debug)]
pub enum Token {
    Skip,
    Text(Vec<u8>),
}

impl LineMatcher {
    pub fn from_line(line: &str) -> LineMatcher {
        let bytes = line.trim().as_bytes();
        let mut last_index = 0;
        let mut cursor = 0;
        let mut tokens = vec![];

        loop {
            match (bytes.get(cursor), bytes.get(cursor + 1), bytes.get(cursor + 2)) {
                (Some(b'.'), Some(b'.'), Some(b'.')) => {
                    if last_index < cursor {
                        tokens.push(Token::Text(bytes[last_index..cursor].to_vec()));
                    }

                    tokens.push(Token::Skip);
                    cursor += 3;
                    last_index = cursor;
                },
                (Some(_), _, _) => {
                    cursor += 1;
                },
                (None, _, _) => {
                    if last_index < cursor {
                        tokens.push(Token::Text(bytes[last_index..cursor].to_vec()));
                    }

                    break;
                },
            }
        }

        match &tokens[..] {
            [Token::Skip, Token::Skip] => LineMatcher::AnyLines,
            _ => LineMatcher::Tokens(tokens),
        }
    }
}

pub fn match_lines(lines: &str, matchers: &Option<Vec<LineMatcher>>) -> Result<(), String> {
    if let Some(matchers) = matchers {
        let lines = lines.lines().map(|line| normalize(line)).collect::<Vec<_>>();
        let mut line_cursor = 0;
        let mut matcher_cursor = 0;

        loop {
            match (matchers.get(matcher_cursor), matchers.get(matcher_cursor + 1)) {
                // meaning less matcher
                (Some(LineMatcher::AnyLines), Some(LineMatcher::AnyLines)) => {
                    matcher_cursor += 1;
                },
                (Some(LineMatcher::AnyLines), Some(LineMatcher::Tokens(s))) => {
                    loop {
                        match lines.get(line_cursor) {
                            Some(line) => {
                                line_cursor += 1;

                                if match_line(s, line) {
                                    matcher_cursor += 2;
                                    break;
                                }
                            },
                            None => {
                                return Err(unexpected_end(matchers, matcher_cursor + 1));
                            },
                        }
                    }
                },
                (Some(LineMatcher::AnyLines), None) => {
                    return Ok(());
                },
                (Some(LineMatcher::Tokens(s)), _) => {
                    match lines.get(line_cursor) {
                        Some(line) => {
                            if match_line(s, line) {
                                matcher_cursor += 1;
                                line_cursor += 1;
                            }

                            else {
                                return Err(no_match(&lines, line_cursor, matchers, matcher_cursor));
                            }
                        },
                        None => {
                            return Err(unexpected_end(matchers, matcher_cursor + 1));
                        },
                    }
                },
                (None, _) => match lines.get(line_cursor) {
                    Some(_) => {
                        return Err(remaining_lines(&lines, line_cursor));
                    },
                    None => {
                        return Ok(());
                    },
                },
            }
        }
    }

    else {
        Ok(())
    }
}

// It's 99% same as `match_lines`. The difference is that it iterates bytes instead of lines.
fn match_line(tokens: &[Token], line: &str) -> bool {
    let bytes = line.as_bytes();
    let mut byte_cursor = 0;
    let mut token_cursor = 0;

    loop {
        match (tokens.get(token_cursor), tokens.get(token_cursor + 1)) {
            (Some(Token::Skip), Some(Token::Skip)) => {
                byte_cursor += 1;
            },
            (Some(Token::Skip), Some(Token::Text(t))) => {
                loop {
                    if bytes.len() < byte_cursor + t.len() {
                        return false;
                    }

                    else {
                        if &bytes[byte_cursor..(byte_cursor + t.len())] == t {
                            token_cursor += 2;
                            byte_cursor += t.len();
                            break;
                        }

                        else {
                            byte_cursor += 1;
                        }
                    }
                }
            },
            (Some(Token::Skip), None) => {
                return true;
            },
            (Some(Token::Text(t)), _) => {
                if bytes.len() < byte_cursor + t.len() {
                    return false;
                }

                else {
                    if &bytes[byte_cursor..(byte_cursor + t.len())] == t {
                        token_cursor += 1;
                        byte_cursor += t.len();
                    }

                    else {
                        return false;
                    }
                }
            },
            (None, _) => {
                if byte_cursor == bytes.len() {
                    return true;
                }

                else {
                    return false;
                }
            },
        }
    }
}

fn normalize(line: &str) -> String {
    remove_ansi_characters(line.trim())
}

fn unexpected_end(matchers: &[LineMatcher], cursor: usize) -> String {
    format!("
Matcher `{}` is not matched.

Matchers:
{}
",
        matchers[cursor],
        highlight_lines(matchers, cursor),
    )
}

fn no_match(lines: &[String], line_cursor: usize, matchers: &[LineMatcher], matcher_cursor: usize) -> String {
    format!("
Line `{}` and matcher `{}` are supposed to match, but they don't.

Lines:
{}

Matchers:
{}
",
        lines[line_cursor],
        matchers[matcher_cursor].to_string(),
        highlight_lines(lines, line_cursor),
        highlight_lines(matchers, matcher_cursor),
    )
}

fn remaining_lines(lines: &[String], cursor: usize) -> String {
    format!("
Line `{}` is not matched.

Lines:
{}
",
        lines[cursor],
        highlight_lines(lines, cursor),
    )
}

fn highlight_lines<T: ToString>(lines: &[T], cursor: usize) -> String {
    highlight_lines_inner(
        lines.iter().map(|line| line.to_string()).collect(),
        cursor,
    )
}

fn highlight_lines_inner(lines: Vec<String>, cursor: usize) -> String {
    let mut result = vec![];
    let start = cursor.max(3) - 3;
    let end = (start + 7).min(lines.len());

    for i in start..end {
        result.push(format!(
            "{} {:>4} | {}",
            if i == cursor { ">>>" } else { "   " },
            i,
            lines[i],
        ));
    }

    result.join("\n")
}
