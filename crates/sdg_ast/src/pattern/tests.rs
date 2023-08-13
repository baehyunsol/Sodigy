use super::PatternErrorKind;

pub fn is_eq_pat_err(p1: &PatternErrorKind, p2: &PatternErrorKind) -> bool {
    p1 == p2  // nothing to do for now
}

// TODO: add test
