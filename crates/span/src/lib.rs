use sodigy_file::File;

pub enum Span {
    File(File),
    Range {
        file: File,

        // start..end
        start: usize,
        end: usize,
    },
    None,
}

impl Span {
    pub fn range(file: File, start: usize, end: usize) -> Self {
        Span::Range { file, start, end }
    }

    pub fn file(file: File) -> Self {
        Span::File(file)
    }
}
