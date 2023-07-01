use crate::session::LocalParseSession;

const MAX_PREVIEW_LEN: usize = 96;

#[derive(Copy, Clone)]
pub struct Span {
    file_no: u32,
    pub index: usize
}

impl Span {

    pub fn new(file_no: u32, index: usize) -> Self {
        Span { file_no, index }
    }

    pub fn dummy() -> Self {
        Span { file_no: u32::MAX, index: usize::MAX }
    }

    pub fn is_dummy(&self) -> bool {
        self.file_no == u32::MAX && self.index == usize::MAX
    }

    // preview of this span for error messages
    pub fn render_err(&self, session: &LocalParseSession) -> String {
        let buffer = session.get_file_raw_content(self.file_no);
        let mut row = 0;
        let mut col = 0;
        let mut curr_col = 0;
        let mut lines = vec![];
        let mut tmp_lines = vec![];

        for (i, c) in buffer.iter().enumerate() {

            if *c == b'\n' {
                lines.push(tmp_lines);
                tmp_lines = vec![];
                curr_col = 0;
            }

            else {
                tmp_lines.push(*c);
                curr_col += 1;
            }

            if self.index == i {
                row = lines.len();
                col = curr_col;
            }

        }

        lines.push(tmp_lines);

        let preview = lines.into_iter().enumerate().map(
            |(index, line)| {
                let marker = if index == row {
                    vec![b'>'; 3]
                } else {
                    vec![b' '; 3]
                };

                let line_no = format!(
                    " {index:08} │ "
                ).as_bytes().to_vec();

                let content = if line.len() > MAX_PREVIEW_LEN {
                    vec![
                        cut_char(&line, MAX_PREVIEW_LEN - 3).to_vec(),
                        vec![b'.'; 3]
                    ].concat()
                } else {
                    line
                };

                vec![
                    marker, line_no, content
                ].concat()
            }
        ).collect::<Vec<Vec<u8>>>();

        let preview_start_index = row.max(4) - 4;
        let preview_end_index = (preview_start_index + 9).min(preview.len());

        let mut preview = preview[preview_start_index..preview_end_index].to_vec();

        while preview.iter().all(|line| line[4] == b'0') {
            preview = preview.iter_mut().map(
                |line| {
                    line.remove(4);

                    line.to_vec()
                }
            ).collect()
        }

        preview = insert_col_markers(preview, col);

        preview.iter().map(|line| String::from_utf8_lossy(line).to_string()).collect::<Vec<String>>().join("\n")
    }

}

fn cut_char(line: &[u8], length: usize) -> Vec<u8> {
    let mut result = Vec::with_capacity(length + 3);

    for (ind, c) in line.iter().enumerate() {
        result.push(*c);

        if ind > length {

            if *c < 128 {
                break;
            }

            else if *c >= 192 {
                result.pop().expect("Internal Compiler Error AE41736");
                break;
            }

        }

    }

    result
}

fn insert_col_markers(lines: Vec<Vec<u8>>, col: usize) -> Vec<Vec<u8>> {

    if col >= MAX_PREVIEW_LEN - 3 {
        return lines;
    }

    let line_no_len = lines[0].iter().position(|&c| c == 0xe2).expect("Internal Compiler Error 538DC83");  // the first byte of `│`
    let highlight_line_index = lines.iter().position(|line| line[0] == b'>').expect("Internal Compiler Error 2B1BA68");

    let upper: Vec<u8> = format!("{}│{}▼", " ".repeat(line_no_len), " ".repeat(col)).as_bytes().to_vec();
    let lower: Vec<u8> = format!("{}│{}▲", " ".repeat(line_no_len), " ".repeat(col)).as_bytes().to_vec();

    vec![
        lines[0..highlight_line_index].to_vec(),
        vec![upper],
        vec![lines[highlight_line_index].clone()],
        vec![lower],
        if highlight_line_index + 1 < lines.len() { lines[(highlight_line_index + 1)..].to_vec() } else { vec![] }
    ].concat()
}