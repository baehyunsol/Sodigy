use crate::session::{DUMMY_FILE_INDEX, LocalParseSession};
use crate::utils::bytes_to_string;
use sdg_hash::{SdgHash, SdgHashResult};

const MAX_PREVIEW_LEN: usize = 96;

#[derive(Copy, Clone, Eq, Ord, PartialEq, PartialOrd)]
pub struct Span {
    /// hash of the name of the file
    file_no: u64,

    /// both indices are inclusive
    pub start: usize,
    pub end: usize,
}

impl Span {
    pub fn new(file_no: u64, start: usize, end: usize) -> Self {
        Span { file_no, start, end }
    }

    #[cfg(test)]
    pub fn first() -> Self {
        Span { file_no: DUMMY_FILE_INDEX, start: 0, end: 0 }
    }

    pub fn dummy() -> Self {
        Span {
            file_no: DUMMY_FILE_INDEX,
            start: usize::MAX,
            end: usize::MIN,
        }
    }

    pub fn is_dummy(&self) -> bool {
        self.file_no == DUMMY_FILE_INDEX && self.start == usize::MAX && self.end == usize::MIN
    }

    /// one must call `.set_ind_and_fileno` after initializing this!
    pub fn dummy_index(start: usize) -> Self {
        Span {
            file_no: DUMMY_FILE_INDEX,
            start,
            end: start,
        }
    }

    pub fn is_dummy_index(&self) -> bool {
        self.file_no == DUMMY_FILE_INDEX && self.start != usize::MAX && self.start == self.end
    }

    #[must_use]
    pub fn set_index(&self, index: usize) -> Self {
        Span {
            start: index,
            end: index,
            ..self.clone()
        }
    }

    #[must_use]
    pub fn set_end(&self, end: usize) -> Self {
        Span {
            end,
            ..self.clone()
        }
    }

    #[must_use]
    pub fn set_len(&self, len: usize) -> Self {
        Span {
            end: self.start + len - 1,
            ..self.clone()
        }
    }

    #[must_use]
    pub fn extend(&self, ex: usize) -> Self {
        Span {
            end: self.end + ex,
            ..self.clone()
        }
    }

    #[must_use]
    pub fn backward(&self, offset: usize) -> Option<Self> {

        if offset > self.start {
            None
        }

        else {
            Some(Span {
                file_no: self.file_no,
                start: self.start - offset,
                end: self.end - offset,
            })
        }

    }

    #[must_use]
    pub fn merge(&self, other: &Span) -> Self {
        assert_eq!(self.file_no, other.file_no);

        Span {
            file_no: self.file_no,
            start: self.start.min(other.start),
            end: self.end.max(other.end),
        }
    }

    #[must_use]
    pub fn forward(&self, offset: usize) -> Self {
        Span {
            file_no: self.file_no,
            start: self.start + offset,
            end: self.end + offset,
        }
    }

    #[must_use]
    pub fn last_character(&self) -> Self {
        Span {
            start: self.end,
            ..self.clone()
        }
    }

    #[must_use]
    pub fn first_character(&self) -> Self {
        Span {
            end: self.start,
            ..self.clone()
        }
    }

    pub fn dump(&self, session: &LocalParseSession) -> String {
        if self.is_dummy() {
            String::from("@@DUMMY_SPAN")
        } else {
            String::from_utf8_lossy(&session.get_file_raw_content(self.file_no)[self.start..(self.end + 1)]).to_string()
        }
    }

    // TODO: dirty code
    /// preview of this span for error messages
    pub fn render_err(&self, session: &LocalParseSession) -> String {
        let buffer = session.get_file_raw_content(self.file_no);
        let file_path = session.get_file_path(self.file_no).as_bytes().to_vec();
        let mut row_start = 0;
        let mut col_start = 0;
        let mut row_end = usize::MAX;
        let mut col_end = usize::MAX;
        let mut lines = vec![];
        let mut curr_line = vec![];

        for (i, c) in buffer.iter().enumerate() {
            if *c == b'\n' {
                lines.push(curr_line);
                curr_line = vec![];
            } else {
                curr_line.push(*c);
            }

            if self.start == i {
                row_start = lines.len();
                col_start = curr_line.len();
            } 

            if self.end == i {
                row_end = lines.len();
                col_end = curr_line.len();
            }
        }

        lines.push(curr_line);

        let preview = lines
            .into_iter()
            .enumerate()
            .map(|(index, line)| {
                let marker = if row_start <= index && index <= row_end {
                    "▸".as_bytes().to_vec()
                } else {
                    b" ".to_vec()
                };

                let line_no = format!(" {:08} │ ", index + 1).as_bytes().to_vec();

                let content = if line.len() > MAX_PREVIEW_LEN {
                    vec![cut_char(&line, MAX_PREVIEW_LEN - 3).to_vec(), vec![b'.'; 3]].concat()
                } else {
                    line
                };

                vec![marker, line_no, content].concat()
            })
            .collect::<Vec<Vec<u8>>>();

        let preview_start_index = row_start.max(4) - 4;
        let preview_end_index = (preview_start_index + 9).min(preview.len());

        let mut preview = preview[preview_start_index..preview_end_index].to_vec();

        while preview.iter().all(
            |line| 
                (line[0] == b' ' && line[2] == b'0')
                || (line[0] == 0xe2 && line[4] == b'0')  // 0xe2 is the first byte of `▸`
                || (line[0] == 0xe2 && line[4] == b'1' && line[5] == b' ')
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

        if preview.len() == 1 && row_start == 0 {
            preview[0] = vec!["▸ 1 │".as_bytes().to_vec(), preview[0][8..].to_vec()].concat()
        }

        preview = insert_col_markers(preview, col_start, col_end, preview_end_index < row_end);
        preview.insert(0, render_pos(file_path, row_start, col_start));

        preview
            .iter()
            .map(|line| bytes_to_string(line))
            .collect::<Vec<String>>()
            .join("\n")
    }
}

fn render_pos(file_path: Vec<u8>, row: usize, col: usize) -> Vec<u8> {
    vec![
        file_path,
        // index starts with 0 in Rust, but with 1 in line_no
        format!(":{}:{col}", row + 1).as_bytes().to_vec(),
    ].concat()
}

fn cut_char(line: &[u8], length: usize) -> Vec<u8> {
    let mut result = Vec::with_capacity(length + 3);

    for (ind, c) in line.iter().enumerate() {
        result.push(*c);

        if ind > length {
            if *c < 128 {
                break;
            } else if *c >= 192 {
                result.pop().expect("Internal Compiler Error A79FBD9EF92");
                break;
            }
        }
    }

    result
}

fn insert_col_markers(lines: Vec<Vec<u8>>, col_start: usize, col_end: usize, too_long_to_render: bool) -> Vec<Vec<u8>> {
    if col_start > MAX_PREVIEW_LEN - 3 || col_end > MAX_PREVIEW_LEN - 3 {
        return lines;
    }

    let line_no_len = if lines[0][0] == 0xe2 {  // the first byte of `│` and `▸` are the same...
        assert_eq!(lines[0][4], b'1', "Internal Compiler Error 5A798E8FF9D");
        4
    } else {
        lines[0]
            .iter()
            .position(|&c| c == 0xe2)  // the first byte of `│`
            .expect("Internal Compiler Error B1E7DE3656E")
    };

    let highlight_line_indices = lines
        .iter().enumerate()
        .filter(|(_, line)| line[0] == 0xe2)
        .map(|(index, _)| index)
        .collect::<Vec<usize>>();

    let highlight_line_start = highlight_line_indices[0];
    let highlight_line_end = highlight_line_indices[highlight_line_indices.len() - 1];

    let upper_pre = " ".repeat(col_start);
    let upper_arr = if highlight_line_indices.len() == 1 {
        "▾".repeat(col_end - col_start + 1)
    } else {
        "▾".repeat(lines[highlight_line_start].len().min(MAX_PREVIEW_LEN - 3) - col_start - line_no_len - 5)
    };

    let upper: Vec<u8> = format!("{}│{upper_pre}{upper_arr}", " ".repeat(line_no_len))
        .as_bytes()
        .to_vec();

    let lower_pre = if highlight_line_indices.len() == 1 {
        " ".repeat(col_start)
    } else {
        String::from(" ")
    };
    let lower_arr = if highlight_line_indices.len() == 1 {
        "▴".repeat(col_end - col_start + 1)
    } else {
        "▴".repeat(col_end)
    };

    let lower: Vec<u8> = format!("{}│{lower_pre}{lower_arr}", " ".repeat(line_no_len))
        .as_bytes()
        .to_vec();

    vec![
        lines[0..highlight_line_start].to_vec(),
        if highlight_line_indices.len() < 2 { vec![] } else { vec![upper] },
        lines[highlight_line_start..(highlight_line_end + 1)].to_vec(),
        if too_long_to_render { vec![] } else { vec![lower] },
        if highlight_line_end + 1 < lines.len() {
            lines[(highlight_line_end + 1)..].to_vec()
        } else {
            vec![]
        },
    ]
    .concat()
}

impl SdgHash for Span {
    fn sdg_hash(&self) -> SdgHashResult {
        // (self.end + 1) in case (self.start == self.end)
        self.file_no.sdg_hash() ^ self.start.sdg_hash() ^ (self.end + 1).sdg_hash()
    }
}
