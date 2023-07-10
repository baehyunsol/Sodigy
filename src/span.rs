use crate::session::LocalParseSession;
use crate::utils::bytes_to_string;

const MAX_PREVIEW_LEN: usize = 96;

#[derive(Copy, Clone)]
pub struct Span {
    file_no: u32,
    pub index: usize,
}

impl Span {
    pub fn new(file_no: u32, index: usize) -> Self {
        Span { file_no, index }
    }

    #[cfg(test)]
    pub fn first() -> Self {
        Span { file_no: 0, index: 0 }
    }

    pub fn dummy() -> Self {
        Span {
            file_no: u32::MAX,
            index: usize::MAX,
        }
    }

    pub fn is_dummy(&self) -> bool {
        self.file_no == u32::MAX && self.index == usize::MAX
    }

    // one must call `.set_ind_and_fileno` after initializing this!
    pub fn dummy_index(index: usize) -> Self {
        Span {
            file_no: u32::MAX,
            index,
        }
    }

    pub fn is_dummy_index(&self) -> bool {
        self.file_no == u32::MAX && self.index != usize::MAX
    }

    // preview of this span for error messages
    pub fn render_err(&self, session: &LocalParseSession) -> String {
        let buffer = session.get_file_raw_content(self.file_no);
        let mut row = 0;
        let mut col = 0;
        let mut lines = vec![];
        let mut curr_line = vec![];

        for (i, c) in buffer.iter().enumerate() {
            if *c == b'\n' {
                lines.push(curr_line);
                curr_line = vec![];
            } else {
                curr_line.push(*c);
            }

            if self.index == i {
                row = lines.len();
                col = curr_line.len();
            }
        }

        lines.push(curr_line);

        let preview = lines
            .into_iter()
            .enumerate()
            .map(|(index, line)| {
                let marker = if index == row {
                    "▸".as_bytes().to_vec()
                } else {
                    b" ".to_vec()
                };

                let line_no = format!(" {index:08} │ ").as_bytes().to_vec();

                let content = if line.len() > MAX_PREVIEW_LEN {
                    vec![cut_char(&line, MAX_PREVIEW_LEN - 3).to_vec(), vec![b'.'; 3]].concat()
                } else {
                    line
                };

                vec![marker, line_no, content].concat()
            })
            .collect::<Vec<Vec<u8>>>();

        let preview_start_index = row.max(4) - 4;
        let preview_end_index = (preview_start_index + 9).min(preview.len());

        let mut preview = preview[preview_start_index..preview_end_index].to_vec();

        while preview.iter().all(
            |line| 
                (line[0] == b' ' && line[2] == b'0')
                || (line[0] == 0xe2 && line[4] == b'0')  // 0xe2 is the first byte of `▸`
        ) {
            preview = preview.iter_mut().map(
                    |line| {
                        if line[0] == b' ' {
                            line.remove(2);
                        }

                        else {
                            line.remove(4);
                        }

                        line.to_vec()
                    }
                ).collect()
        }

        if preview.len() == 1 && row == 0 {
            preview[0] = vec!["▸ 0 │".as_bytes().to_vec(), preview[0][8..].to_vec()].concat()
        }

        preview = insert_col_markers(preview, col);

        preview
            .iter()
            .map(|line| bytes_to_string(line))
            .collect::<Vec<String>>()
            .join("\n")
    }
}

fn cut_char(line: &[u8], length: usize) -> Vec<u8> {
    let mut result = Vec::with_capacity(length + 3);

    for (ind, c) in line.iter().enumerate() {
        result.push(*c);

        if ind > length {
            if *c < 128 {
                break;
            } else if *c >= 192 {
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

    let line_no_len = if lines.len() == 1 {
        assert_eq!(lines[0][4], b'0', "Internal Compiler Error 00CDDBE");
        4
    } else {
        lines[0]
            .iter()
            .position(|&c| c == 0xe2)  // the first byte of `│`
            .expect("Internal Compiler Error 538DC83")
    };

    let highlight_line_index = lines
        .iter()
        .position(|line| line[0] == 0xe2)
        .expect("Internal Compiler Error 2B1BA68");  // the first byte of `▸`

    let upper: Vec<u8> = format!("{}│{}▾", " ".repeat(line_no_len), " ".repeat(col))
        .as_bytes()
        .to_vec();
    let lower: Vec<u8> = format!("{}│{}▴", " ".repeat(line_no_len), " ".repeat(col))
        .as_bytes()
        .to_vec();

    vec![
        lines[0..highlight_line_index].to_vec(),
        vec![upper],
        vec![lines[highlight_line_index].clone()],
        vec![lower],
        if highlight_line_index + 1 < lines.len() {
            lines[(highlight_line_index + 1)..].to_vec()
        } else {
            vec![]
        },
    ]
    .concat()
}
