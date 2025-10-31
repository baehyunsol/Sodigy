// TODO: maybe rewrite this entire thing in Sodigy?

#[derive(Clone, Copy, Debug)]
pub struct Config {
    pub single_line_paren_limit: usize,
    pub max_line_width: usize,
    pub indent: usize,
    pub ignore_quote: bool,
}

impl Default for Config {
    fn default() -> Self {
        Config {
            single_line_paren_limit: 20,
            max_line_width: 80,
            indent: 4,
            ignore_quote: false,
        }
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
enum State {
    Text,
    Comment {
        escape_to_slp: Option<usize>,
    },
    String {
        delim: u8,
        escape_to_slp: Option<usize>,
    },
    SingleLineParen(usize),
    Corrupted,
    Done,
}

pub struct Context {
    state: State,
    input: Vec<u8>,
    cursor: usize,
    output: Vec<u8>,
    stack: Vec<u8>,
    config: Config,
}

impl Context {
    pub fn new(input: Vec<u8>) -> Self {
        Context::with_config(input, Config::default())
    }

    pub fn with_config(input: Vec<u8>, config: Config) -> Self {
        Context {
            state: State::Text,
            input,
            cursor: 0,
            output: vec![],
            stack: vec![],
            config,
        }
    }

    pub fn step_all(&mut self) {
        while self.state != State::Done {
            self.step();
        }
    }

    pub fn output(&self) -> &[u8] {
        &self.output
    }

    fn step(&mut self) {
        match self.state {
            State::Text => match (self.input.get(self.cursor), self.input.get(self.cursor + 1)) {
                (Some(b'/'), Some(b'/')) => {
                    self.output.push(b'/');
                    self.output.push(b'/');
                    self.cursor += 2;
                    self.state = State::Comment { escape_to_slp: None };
                },
                (Some(b'#'), _) => {
                    self.output.push(b'#');
                    self.cursor += 1;
                    self.state = State::Comment { escape_to_slp: None };
                },
                (Some(b @ (b'{' | b'[' | b'(')), _) => {
                    let matched_b = match_paren(*b);
                    let n = self.look_ahead(
                        *b,
                        matched_b,
                        self.cursor,
                        self.config.single_line_paren_limit + 1,
                    );
                    self.output.push(*b);
                    self.stack.push(matched_b);
                    self.cursor += 1;

                    if n < self.config.single_line_paren_limit {
                        self.state = State::SingleLineParen(self.stack.len() - 1);
                    }

                    else {
                        self.output.push(b'\n');
                        self.push_indent();
                    }
                },
                (Some(b @ (b'}' | b']' | b')')), _) => {
                    let b = *b;
                    let matched_b = self.stack.pop();

                    // it has a matched delim, so we can apply the prettifier-rules
                    if Some(b) == matched_b {
                        self.remove_trailing_whitespace();

                        if !self.ends_with_new_line() {
                            self.output.push(b'\n');
                            self.push_indent();
                        }
                    }

                    else {
                        self.state = State::Corrupted;
                    }

                    self.output.push(b);
                    self.cursor += 1;
                },
                (Some(b','), _) => {
                    self.output.push(b',');
                    self.cursor += 1;

                    // inside a group
                    if !self.stack.is_empty() {
                        if self.output_line_length(self.config.max_line_width + 1) > self.config.max_line_width {
                            self.output.push(b'\n');
                            self.push_indent();
                        }

                        else {
                            self.output.push(b' ');
                        }
                    }
                },
                (Some(b @ (b' ' | b'\n' | b'\t')), _) => {
                    // inside a group
                    if !self.stack.is_empty() {
                        if !self.ends_with_whitespace() && !self.ends_with_new_line() {
                            self.output.push(b' ');
                        }
                    }

                    else {
                        self.output.push(*b);
                    }

                    self.cursor += 1;
                },
                (Some(b @ (b'"' | b'\'')), _) if !self.config.ignore_quote => {
                    self.output.push(*b);
                    self.cursor += 1;
                    self.state = State::String {
                        delim: *b,
                        escape_to_slp: None,
                    };
                },
                (Some(b), _) => {
                    self.output.push(*b);
                    self.cursor += 1;
                },
                (None, _) => {
                    self.state = State::Done;
                },
            },
            State::Comment { escape_to_slp } => match self.input.get(self.cursor) {
                Some(b'\n') => {
                    self.output.push(b'\n');
                    self.push_indent();
                    self.cursor += 1;

                    if let Some(m) = escape_to_slp {
                        self.state = State::SingleLineParen(m);
                    }

                    else {
                        self.state = State::Text;
                    }
                },
                Some(b) => {
                    self.output.push(*b);
                    self.cursor += 1;
                },
                None => {
                    self.state = State::Done;
                },
            },
            State::String { delim, escape_to_slp } => match (self.input.get(self.cursor), self.input.get(self.cursor + 1)) {
                (Some(b'\\'), Some(b)) => {
                    self.output.push(b'\\');
                    self.output.push(*b);
                    self.cursor += 2;
                },
                (Some(b), _) if *b == delim => {
                    self.output.push(*b);
                    self.cursor += 1;

                    if let Some(m) = escape_to_slp {
                        self.state = State::SingleLineParen(m);
                    }

                    else {
                        self.state = State::Text;
                    }
                },
                (Some(b), _) => {
                    self.output.push(*b);
                    self.cursor += 1;
                },
                (None, _) => {
                    self.state = State::Done;
                },
            },
            State::Corrupted => match self.input.get(self.cursor) {
                Some(b) => {
                    self.output.push(*b);
                    self.cursor += 1;
                },
                None => {
                    self.state = State::Done;
                },
            },
            State::SingleLineParen(s) => match (self.input.get(self.cursor), self.input.get(self.cursor + 1)) {
                (Some(b'/'), Some(b'/')) => {
                    self.output.push(b'/');
                    self.output.push(b'/');
                    self.cursor += 2;
                    self.state = State::Comment {
                        escape_to_slp: Some(s),
                    };
                },
                (Some(b'#'), _) => {
                    self.output.push(b'#');
                    self.cursor += 1;
                    self.state = State::Comment {
                        escape_to_slp: Some(s),
                    };
                },
                (Some(b @ (b'"' | b'\'')), _) => {
                    self.output.push(*b);
                    self.cursor += 1;
                    self.state = State::String {
                        delim: *b,
                        escape_to_slp: Some(s),
                    };
                },
                (Some(b' ' | b'\n' | b'\t'), _) => {
                    if !self.ends_with_whitespace() && !self.ends_with_new_line() {
                        self.output.push(b' ');
                    }

                    self.cursor += 1;
                },
                (Some(b @ (b'{' | b'[' | b'(')), _) => {
                    self.output.push(*b);
                    self.stack.push(match_paren(*b));
                    self.cursor += 1;
                },
                (Some(b @ (b'}' | b']' | b')')), _) => {
                    let matched_b = self.stack.pop();
                    self.output.push(*b);
                    self.cursor += 1;

                    if self.stack.len() == s {
                        self.state = State::Text;
                    }

                    if Some(*b) != matched_b {
                        self.state = State::Corrupted;
                    }
                },
                (Some(b), _) => {
                    self.output.push(*b);
                    self.cursor += 1;
                },
                (None, _) => {
                    self.state = State::Done;
                },
            },
            State::Done => {},
        }
    }

    fn push_indent(&mut self) {
        for _ in 0..(self.stack.len() * self.config.indent) {
            self.output.push(b' ');
        }
    }

    fn look_ahead(&self, skip: u8, target: u8, start: usize, max_search: usize) -> usize {
        let mut stack = 0;

        for i in start..(start + max_search) {
            match self.input.get(i) {
                Some(b) if *b == target => {
                    stack -= 1;

                    if stack == 0 {
                        return i - start;
                    }
                },
                Some(b) if *b == skip => {
                    stack += 1;
                },
                None => {
                    return max_search;
                },
                _ => {},
            }
        }

        max_search
    }

    fn remove_trailing_whitespace(&mut self) {
        while let Some(b) = self.output.pop() {
            if b != b' ' && b != b'\t' {
                self.output.push(b);
                break;
            }
        }
    }

    fn ends_with_new_line(&self) -> bool {
        match self.output.last() {
            Some(b'\n') => true,
            _ => false,
        }
    }

    fn ends_with_whitespace(&self) -> bool {
        match self.output.last() {
            Some(b' ' | b'\t') => true,
            _ => false,
        }
    }

    fn output_line_length(&self, max_search: usize) -> usize {
        for i in 1..(max_search + 1) {
            if i > self.output.len() {
                return i;
            }

            if self.output[self.output.len() - i] == b'\n' {
                return i - 1;
            }
        }

        max_search
    }
}

fn match_paren(b: u8) -> u8 {
    match b {
        b'{' => b'}',
        b'[' => b']',
        b'(' => b')',
        _ => unreachable!(),
    }
}
