use crate::Span;
use std::cmp::Ordering;

impl Ord for Span {
    fn cmp(&self, other: &Self) -> Ordering {
        match (self, other) {
            // `Span::None` is the smallest
            (Span::None, Span::None) => Ordering::Equal,
            (Span::None, _) => Ordering::Less,
            (_, Span::None) => Ordering::Greater,

            // `Span::Std` is the next smallest
            (Span::Std, Span::Std) => Ordering::Equal,
            (Span::Std, _) => Ordering::Less,
            (_, Span::Std) => Ordering::Greater,

            // `Span::Prelude` is the next smallest
            (Span::Prelude(p1), Span::Prelude(p2)) => p1.cmp(p2),
            (Span::Prelude(_), _) => Ordering::Less,
            (_, Span::Prelude(_)) => Ordering::Greater,

            // `Span::Lib` is the next smallest
            (Span::Lib, Span::Lib) => Ordering::Equal,
            (Span::Lib, _) => Ordering::Less,
            (_, Span::Lib) => Ordering::Greater,

            // `Span::Poly` is the next smallest
            (Span::Poly { name: name1, kind: kind1 }, Span::Poly { name: name2, kind: kind2 }) => match name1.cmp(name2) {
                Ordering::Equal => kind1.cmp(kind2),
                c => c,
            },
            (Span::Poly { .. }, _) => Ordering::Less,
            (_, Span::Poly { .. }) => Ordering::Greater,

            // Then, it compares files.
            (
                Span::File(file1) | Span::Range { file: file1, .. } | Span::Derived { file: file1, .. } | Span::Eof(file1),
                Span::File(file2) | Span::Range { file: file2, .. } | Span::Derived { file: file2, .. } | Span::Eof(file2),
            ) if file1 != file2 => file1.cmp(file2),
            // If the 2 spans are pointing to the same file, it compares the indexes.
            // `Span::File` is treated like the start of a file, and `Span::Eof` is of course the end of the file.
            (Span::File(_), Span::File(_)) |
            (Span::Eof(_), Span::Eof(_)) => Ordering::Equal,
            (Span::File(_), Span::Range { .. } | Span::Derived { .. } | Span::Eof(_)) |
            (Span::Range { .. } | Span::Derived { .. }, Span::Eof(_)) => Ordering::Less,
            (Span::Eof(_), Span::Range { .. } | Span::Derived { .. } | Span::File(_)) |
            (Span::Range { .. } | Span::Derived { .. }, Span::File(_)) => Ordering::Greater,

            // `Span::Derived` is treated like `Span::Range`. But if they're pointing to the exact same token,
            // `Span::Range` is greater.
            (
                Span::Range { start: start1, end: end1, .. } | Span::Derived { start: start1, end: end1, .. },
                Span::Range { start: start2, end: end2, .. } | Span::Derived { start: start2, end: end2, .. },
            ) => match start1.cmp(start2) {
                c @ (Ordering::Less | Ordering::Greater) => c,
                Ordering::Equal => match end1.cmp(end2) {
                    c @ (Ordering::Less | Ordering::Greater) => c,
                    Ordering::Equal => match (self, other) {
                        (Span::Range { .. }, Span::Derived { .. }) => Ordering::Greater,
                        (Span::Derived { .. }, Span::Range { .. }) => Ordering::Less,
                        _ => Ordering::Equal,
                    },
                },
            },
        }
    }
}

// It's used to sort error messages.
impl PartialOrd for Span {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        Some(self.cmp(other))
    }
}
