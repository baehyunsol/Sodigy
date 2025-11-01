use crate::Span;
use sodigy_file::File;
use std::collections::hash_map::{Entry, HashMap};

pub struct Session {
    intermediate_dir: String,
    file_paths: HashMap<File, String>,

    // Spans only have byte offset, but we want row and col indexes.
    // So the session remembers the line breaks.
    line_breaks: HashMap<File, Vec<usize>>,
}

impl Session {
    pub fn new(intermediate_dir: &str) -> Self {
        Session {
            intermediate_dir: intermediate_dir.to_string(),
            file_paths: HashMap::new(),
            line_breaks: HashMap::new(),
        }
    }

    pub fn get_bytes(&mut self, span: Span) -> Option<Vec<u8>> {
        match span.get_file() {
            Some(file) => match file.read_bytes(&self.intermediate_dir) {
                Ok(Some(bytes)) => {
                    if let Entry::Vacant(e) = self.line_breaks.entry(file) {
                        e.insert(bytes.iter().enumerate().filter(
                            |(_, b)| **b == b'\n'
                        ).map(
                            |(i, _)| i
                        ).collect());
                    }

                    Some(bytes)
                },
                _ => None,
            },
            None => None,
        }
    }

    pub fn get_path(&mut self, span: Span) -> Option<String> {
        match span.get_file() {
            Some(file) => match self.file_paths.entry(file) {
                Entry::Occupied(e) => Some(e.get().to_string()),
                Entry::Vacant(e) => match file.get_path(&self.intermediate_dir) {
                    Ok(Some((_, file_path))) => {
                        e.insert(file_path.to_string());
                        Some(file_path.to_string())
                    },
                    _ => None,
                },
            },
            None => None,
        }
    }

    // rect: [left, top, right, bottom] -> all inclusive
    pub fn get_rect(&mut self, span: Span) -> Option<(usize, usize, usize, usize)> {
        match span {
            Span::Range { file, start, end } => {
                let line_breaks = match self.line_breaks.entry(file) {
                    Entry::Occupied(e) => e.get().to_vec(),
                    Entry::Vacant(e) => match file.read_bytes(&self.intermediate_dir) {
                        Ok(Some(bytes)) => {
                            let line_breaks = bytes.iter().enumerate().filter(
                                |(_, b)| **b == b'\n'
                            ).map(
                                |(i, _)| i
                            ).collect::<Vec<_>>();
                            e.insert(line_breaks.clone());
                            line_breaks
                        },
                        _ => {
                            return None;
                        },
                    },
                };

                Some(get_rect(&line_breaks, start, end))
            },
            _ => None,
        }
    }
}

fn get_rect(line_breaks: &[usize], start: usize, end: usize) -> (usize, usize, usize, usize) {
    // TODO: I haven't tested this logic.
    let start_line_no = match line_breaks.binary_search(&start) {
        Ok(n) => n + 1,
        Err(n) => n,
    };
    let end_line_no = match line_breaks.binary_search(&end) {
        Ok(n) => n,
        Err(n) => n,
    };

    let (left, right) = if start_line_no == end_line_no {
        if start_line_no == 0 {
            (start, end.max(1) - 1)
        }

        else {
            let start_x = start - line_breaks[start_line_no - 1] + 1;
            let end_x = end - line_breaks[end_line_no - 1];
            (start_x, end_x)
        }
    } else {
        let start_x = if start_line_no == 0 {
            start
        } else {
            start - line_breaks[start_line_no - 1]
        };
        let end_x = end - line_breaks[end_line_no - 1] - 1;
        let mut max_line_width = 0;

        for i in start_line_no..end_line_no {
            let line_width = line_breaks[i] - if i == 0 { 0 } else { line_breaks[i - 1] + 1 };

            // it doesn't count '\n' at the end
            max_line_width = max_line_width.max(line_width.max(1) - 1);
        }

        (0, max_line_width.max(start_x).max(end_x))
    };

    (left, start_line_no, right, end_line_no)
}

#[cfg(test)]
mod tests {
    use super::get_rect;

    #[test]
    fn get_rect_test() {
        // @  @  @  @  \n
        // _  _  _  \n
        assert_eq!(
            get_rect(&[4, 8], 0, 4),
            (0, 0, 3, 0),
        );

        // @  @  @  @  \n
        // @  _  _  \n
        assert_eq!(
            get_rect(&[4, 8], 0, 6),
            (0, 0, 3, 1),
        );

        // _  _  _  _  \n
        // _  @  @  \n
        // @  _  _  \n
        assert_eq!(
            get_rect(&[4, 8, 12], 6, 10),
            (0, 1, 2, 2),
        );

        // _  _  _  _  \n
        // _  @  @  \n
        // @  @  @  @  \n
        // @  _  _  \n
        assert_eq!(
            get_rect(&[4, 8, 13, 17], 6, 15),
            (0, 1, 3, 3),
        );

        // _  @  @  @  \n
        // @  @  @  \n
        // @  @  @  @  \n
        // @  _  _  \n
        assert_eq!(
            get_rect(&[4, 8, 13, 17], 1, 15),
            (0, 0, 3, 3),
        );
        // _  _  @  @  \n
        // @  @  @  \n
        // @  _  _  _  \n
        // _  _  _  \n
        assert_eq!(
            get_rect(&[4, 8, 13, 17], 2, 10),
            (0, 0, 3, 2),
        );
    }
}
