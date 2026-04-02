use crate::Span;
use std::cmp::Ordering;

impl Ord for Span {
    fn cmp(&self, other: &Self) -> Ordering {
        match (self.file(), self.get_bounds(), other.file(), other.get_bounds()) {
            (Some(f1), Some((s1, e1)), Some(f2), Some((s2, e2))) => match f1.cmp(&f2) {
                Ordering::Equal => match s1.cmp(&s2) {
                    Ordering::Equal => match e1.cmp(&e2) {
                        Ordering::Equal => match (self, other) {
                            (Span::Range(_), Span::Range(_)) => Ordering::Equal,

                            // In these branches, the orders don't have much meaning.
                            // It's sufficient if it's deterministic.
                            (Span::Monomorphize { id: id1, span: span1 }, Span::Monomorphize { id: id2, span: span2 }) => match id1.cmp(id2) {
                                Ordering::Equal => span1.cmp(span2),
                                o => o,
                            },
                            (Span::Monomorphize { .. }, _) => Ordering::Greater,
                            (Span::Derived { kind: kind1, span: span1 }, Span::Derived { kind: kind2, span: span2 }) => match kind1.cmp(kind2) {
                                Ordering::Equal => span1.cmp(span2),
                                o => o,
                            },
                            (Span::Derived { .. }, _) => Ordering::Greater,
                            _ => unreachable!(),
                        },
                        o => o,
                    },
                    o => o,
                },
                o => o,
            },
            // dummy span is the smallest
            (Some(_), Some(_), None, None) => Ordering::Greater,
            (None, None, Some(_), Some(_)) => Ordering::Less,
            (None, None, None, None) => match (self, other) {
                (Span::None, Span::None) => Ordering::Equal,
                p => panic!("TODO: {p:?}"),
            },
            _ => unreachable!(),
        }
    }
}

// It's used to sort error messages.
impl PartialOrd for Span {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        Some(self.cmp(other))
    }
}
