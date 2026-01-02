pub struct IndentedLines {
    indent: usize,
    lines: Vec<(String, usize /* indent */)>,
}

impl IndentedLines {
    pub fn new() -> IndentedLines {
        IndentedLines {
            indent: 0,
            lines: vec![],
        }
    }

    pub fn inc_indent(&mut self) {
        self.indent += 1;
    }

    pub fn dec_indent(&mut self) {
        self.indent -= 1;
    }

    pub fn break_line(&mut self) {
        self.lines.push((String::new(), self.indent));
    }

    pub fn push(&mut self, code: &str) {
        match self.lines.last_mut() {
            Some((line, _)) => {
                *line = format!("{line}{code}");
            },
            None => {
                self.lines.push((code.to_string(), self.indent));
            },
        }
    }

    pub fn dump(&self) -> String {
        let mut lines = self.lines.iter().map(
            |(line, indent)| format!("{}{line}", "    ".repeat(*indent))
        ).collect::<Vec<_>>();
        lines.join("\n")
    }
}
