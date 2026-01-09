use super::{Constructor, NameBinding};
use sodigy_mir::MatchArm;
use sodigy_number::InternedNumber;
use std::cmp::Ordering;

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum LiteralType {
    Int,
    Number,
    Byte,
    Char,
}

impl LiteralType {
    pub fn is_int_like(&self) -> bool {
        match self {
            LiteralType::Int | LiteralType::Byte | LiteralType::Char => true,
            LiteralType::Number => false,
        }
    }
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct Range {
    pub r#type: LiteralType,
    pub lhs: Option<InternedNumber>,
    pub lhs_inclusive: bool,
    pub rhs: Option<InternedNumber>,
    pub rhs_inclusive: bool,
}

// [
//     (1,    arm-0),
//     (2,    arm-1),
//     (..0,  arm-2),
//     (5..,  arm-3),
//     (10.., arm-4),
//     (_,    arm-5),
// ]
// ->
// [
//     (..0,   [arm-2, arm-5]),
//     (1,     [arm-0, arm-5]),
//     (2,     [arm-1, arm-5]),
//     (3..5,  [arm-5]),
//     (5..10, [arm-3, arm-5]),
//     (10..,  [arm-4, arm-5]),
// ]
pub fn remove_overlaps<T: Clone + Merge>(mut branches: Vec<(Range, T)>) -> Vec<(Range, T)> {
    loop {
        let mut result = Vec::with_capacity(branches.len());
        let mut has_overlap = false;
        branches.sort_by(
            |(range1, _), (range2, _)| match (&range1.lhs, &range2.lhs) {
                (None, None) => Ordering::Equal,
                (None, Some(_)) => Ordering::Less,
                (Some(_), None) => Ordering::Greater,
                (Some(a), Some(b)) => match a.cmp(b) {
                    c @ (Ordering::Less | Ordering::Greater) => c,
                    Ordering::Equal => match (range1.lhs_inclusive, range2.lhs_inclusive) {
                        (true, true) | (false, false) => Ordering::Equal,
                        (true, false) => Ordering::Less,
                        (false, true) => Ordering::Greater,
                    },
                },
            }
        );

        let mut i = 0;

        loop {
            let (a, b) = match (branches.get(i), branches.get(i + 1)) {
                (Some(a), Some(b)) => (a, b),
                (Some(a), None) => {
                    result.push(a.clone());
                    break;
                },
                _ => break,
            };

            let splits = split_to_non_overlapping_ranges(a, b);

            if splits.overlap.is_none() {
                result.push(a.clone());
                i += 1;
            }

            else {
                for part in [
                    splits.overlap.clone(),
                    splits.a_left.clone(),
                    splits.a_right.clone(),
                    splits.b_left.clone(),
                    splits.b_right.clone(),
                ] {
                    if let Some(x) = part {
                        result.push(x);
                    }
                }

                has_overlap = true;
                i += 2;
            }
        }

        if !has_overlap {
            return result;
        }

        else {
            branches = result;
        }
    }
}

pub fn merge_conditions(branches: Vec<(Range, (Vec<(usize, &MatchArm)>, Vec<NameBinding>))>) -> Vec<(Constructor, (Vec<(usize, &MatchArm)>, Vec<NameBinding>))> {
    let mut result = Vec::with_capacity(branches.len());

    'outer_loop: for (range, arms) in branches.into_iter() {
        for (constructor, arms_r) in result.iter_mut() {
            if are_arms_eq(&arms, arms_r) {
                match constructor {
                    Constructor::Or(constructors) => {
                        constructors.push(Constructor::Range(range));
                    },
                    Constructor::Range(_) => {
                        *constructor = Constructor::Or(vec![constructor.clone(), Constructor::Range(range)]);
                    },
                    _ => unreachable!(),
                }

                continue 'outer_loop;
            }
        }

        result.push((Constructor::Range(range), arms));
    }

    result
}

#[derive(Clone, Debug, Eq, PartialEq)]
struct Splits<T> {
    a_left: Option<(Range, T)>,
    a_right: Option<(Range, T)>,
    overlap: Option<(Range, T)>,
    b_left: Option<(Range, T)>,
    b_right: Option<(Range, T)>,
}

fn split_to_non_overlapping_ranges<T: Clone + Merge>((a_range, a_val): &(Range, T), (b_range, b_val): &(Range, T)) -> Splits<T> {
    // max(a.lhs, b.lhs) is lhs of intersection
    let (lhs, lhs_inclusive) = match (&a_range.lhs, &b_range.lhs) {
        // if lhs is None, that's -inf
        (None, _) => (b_range.lhs.clone(), b_range.lhs_inclusive),
        (_, None) => (a_range.lhs.clone(), a_range.lhs_inclusive),
        (Some(a_lhs), Some(b_lhs)) => match a_lhs.cmp(b_lhs) {
            Ordering::Greater => (a_range.lhs.clone(), a_range.lhs_inclusive),
            Ordering::Less => (b_range.lhs.clone(), b_range.lhs_inclusive),
            Ordering::Equal => {
                if !a_range.lhs_inclusive {
                    (a_range.lhs.clone(), a_range.lhs_inclusive)
                } else {
                    (b_range.lhs.clone(), b_range.lhs_inclusive)
                }
            },
        },
    };

    // min(a.rhs, b.rhs) is rhs of intersection
    let (rhs, rhs_inclusive) = match (&a_range.rhs, &b_range.rhs) {
        // if rhs is None, that's +inf
        (None, _) => (b_range.rhs.clone(), b_range.rhs_inclusive),
        (_, None) => (a_range.rhs.clone(), a_range.rhs_inclusive),
        (Some(a_rhs), Some(b_rhs)) => match a_rhs.cmp(b_rhs) {
            Ordering::Greater => (b_range.rhs.clone(), b_range.rhs_inclusive),
            Ordering::Less => (a_range.rhs.clone(), a_range.rhs_inclusive),
            Ordering::Equal => {
                if !a_range.rhs_inclusive {
                    (a_range.rhs.clone(), a_range.rhs_inclusive)
                } else {
                    (b_range.rhs.clone(), b_range.rhs_inclusive)
                }
            },
        },
    };

    let overlap = match (&lhs, &rhs) {
        (None, _) | (_, None) => Some(Range { r#type: a_range.r#type, lhs, lhs_inclusive, rhs, rhs_inclusive }),
        (Some(lhs_n), Some(rhs_n)) => match lhs_n.cmp(rhs_n) {
            Ordering::Greater => None,
            Ordering::Less => {
                // if it's Int, `{ lhs: a, lhs_inclusive: false, rhs: a + 1, rhs_inclusive: false }` is an empty range!
                if a_range.r#type.is_int_like() && !lhs_inclusive && !rhs_inclusive && &lhs_n.add_one() == rhs_n {
                    None
                }

                else {
                    Some(Range { r#type: a_range.r#type, lhs, lhs_inclusive, rhs, rhs_inclusive })
                }
            },
            Ordering::Equal => {
                if lhs_inclusive && rhs_inclusive {
                    Some(Range { r#type: a_range.r#type, lhs, lhs_inclusive, rhs, rhs_inclusive })
                }

                else {
                    None
                }
            },
        },
    };

    if let Some(overlap) = overlap {
        let (a_left, a_right) = get_left_and_right(a_range, &overlap);
        let (b_left, b_right) = get_left_and_right(b_range, &overlap);

        Splits {
            overlap: Some((overlap, a_val.merge(b_val))),
            a_left: a_left.map(|r| (r, a_val.clone())),
            a_right: a_right.map(|r| (r, a_val.clone())),
            b_left: b_left.map(|r| (r, b_val.clone())),
            b_right: b_right.map(|r| (r, b_val.clone())),
        }
    }

    else {
        Splits {
            overlap: None,
            a_left: Some((a_range.clone(), a_val.clone())),
            a_right: None,
            b_left: Some((b_range.clone(), b_val.clone())),
            b_right: None,
        }
    }
}

fn get_left_and_right(super_range: &Range, sub_range: &Range) -> (Option<Range>, Option<Range>) {
    // left is super.lhs..sub.lhs
    let left = match (&super_range.lhs, &sub_range.lhs) {
        (_, None) => None,
        (None, _) => Some(Range {
            r#type: super_range.r#type,
            lhs: super_range.lhs.clone(),
            lhs_inclusive: super_range.lhs_inclusive,
            rhs: sub_range.lhs.clone(),
            rhs_inclusive: !sub_range.lhs_inclusive,
        }),
        (Some(super_lhs), Some(sub_lhs)) => match super_lhs.cmp(sub_lhs) {
            Ordering::Less => Some(Range {
                r#type: super_range.r#type,
                lhs: super_range.lhs.clone(),
                lhs_inclusive: super_range.lhs_inclusive,
                rhs: sub_range.lhs.clone(),
                rhs_inclusive: !sub_range.lhs_inclusive,
            }),
            Ordering::Greater => unreachable!(),
            Ordering::Equal => {
                if super_range.lhs_inclusive && !sub_range.lhs_inclusive {
                    Some(Range {
                        r#type: super_range.r#type,
                        lhs: super_range.lhs.clone(),
                        lhs_inclusive: true,
                        rhs: sub_range.lhs.clone(),
                        rhs_inclusive: true,
                    })
                }

                else {
                    None
                }
            },
        },
    };

    // right is sub.rhs..super.rhs
    let right = match (&sub_range.rhs, &super_range.rhs) {
        (None, _) => None,
        (_, None) => Some(Range {
            r#type: super_range.r#type,
            lhs: sub_range.rhs.clone(),
            lhs_inclusive: !sub_range.rhs_inclusive,
            rhs: super_range.rhs.clone(),
            rhs_inclusive: super_range.rhs_inclusive,
        }),
        (Some(sub_rhs), Some(super_rhs)) => match sub_rhs.cmp(super_rhs) {
            Ordering::Less => Some(Range {
                r#type: super_range.r#type,
                lhs: sub_range.rhs.clone(),
                lhs_inclusive: !sub_range.rhs_inclusive,
                rhs: super_range.rhs.clone(),
                rhs_inclusive: super_range.rhs_inclusive,
            }),
            Ordering::Greater => unreachable!(),
            Ordering::Equal => {
                if !sub_range.rhs_inclusive && super_range.rhs_inclusive {
                    Some(Range {
                        r#type: super_range.r#type,
                        lhs: sub_range.rhs.clone(),
                        lhs_inclusive: true,
                        rhs: super_range.rhs.clone(),
                        rhs_inclusive: true,
                    })
                }

                else {
                    None
                }
            },
        },
    };

    (left, right)
}

pub trait Merge {
    fn merge(&self, other: &Self) -> Self;
}

impl Merge for (Vec<(usize, &MatchArm)>, Vec<NameBinding>) {
    fn merge(&self, other: &Self) -> Self {
        let mut arms = vec![
            self.0.clone(),
            other.0.clone(),
        ].concat();
        let mut name_bindings = vec![
            self.1.clone(),
            other.1.clone(),
        ].concat();

        // 1. We have to make sure that `arms` are sorted because earlier arms must be matched first.
        // 2. It's nice to keep `name_bindings` sorted because it makes comparing `name_bindings` easier.
        arms.sort_by_key(|(id, _)| *id);
        name_bindings.sort_by_key(|name_binding| name_binding.id);

        (arms, name_bindings)
    }
}

#[cfg(test)]
mod tests {
    use super::{
        LiteralType,
        Merge,
        Range,
        get_left_and_right,
        split_to_non_overlapping_ranges,
    };
    use sodigy_number::{InternedNumber, InternedNumberValue};

    // for tests
    impl Merge for i32 {
        fn merge(&self, other: &i32) -> i32 {
            *self + *other
        }
    }

    fn into_range((lhs, lhs_inclusive, rhs, rhs_inclusive): (Option<i32>, bool, Option<i32>, bool)) -> Range {
        Range {
            r#type: LiteralType::Int,
            lhs: lhs.map(|n| InternedNumber { value: InternedNumberValue::SmallInt(n as i64), is_integer: true }),
            lhs_inclusive,
            rhs: rhs.map(|n| InternedNumber { value: InternedNumberValue::SmallInt(n as i64), is_integer: true }),
            rhs_inclusive,
        }
    }

    #[test]
    fn left_right_test() {
        for (super_range, sub_range, left, right) in [
            (
                (Some(0), true, Some(100), true),
                (Some(0), true, Some(50), true),
                None,
                Some((Some(50), false, Some(100), true)),
            ),
            (
                (Some(0), true, Some(100), true),
                (Some(50), true, Some(100), true),
                Some((Some(0), true, Some(50), false)),
                None,
            ),
            (
                (Some(0), true, Some(100), true),
                (Some(30), true, Some(60), true),
                Some((Some(0), true, Some(30), false)),
                Some((Some(60), false, Some(100), true)),
            ),
            (
                (Some(0), true, Some(100), true),
                (Some(0), true, Some(100), true),
                None,
                None,
            ),
            (
                (Some(0), true, Some(100), true),
                (Some(0), true, Some(100), false),
                None,
                Some((Some(100), true, Some(100), true)),
            ),
            (
                (Some(0), true, Some(100), true),
                (Some(0), false, Some(100), true),
                Some((Some(0), true, Some(0), true)),
                None,
            ),
            (
                (Some(0), true, Some(100), true),
                (Some(0), false, Some(100), false),
                Some((Some(0), true, Some(0), true)),
                Some((Some(100), true, Some(100), true)),
            ),
            (
                (None, true, Some(100), true),
                (Some(0), true, Some(30), false),
                Some((None, true, Some(0), false)),
                Some((Some(30), true, Some(100), true)),
            ),
            (
                (None, true, Some(100), true),
                (Some(0), false, Some(30), false),
                Some((None, true, Some(0), true)),
                Some((Some(30), true, Some(100), true)),
            ),
            (
                (None, true, None, true),
                (Some(0), true, Some(30), true),
                Some((None, true, Some(0), false)),
                Some((Some(30), false, None, true)),
            ),
            (
                (None, true, None, true),
                (Some(0), false, Some(30), false),
                Some((None, true, Some(0), true)),
                Some((Some(30), true, None, true)),
            ),
        ] {
            let super_range = into_range(super_range);
            let sub_range = into_range(sub_range);
            let left_answer = left.map(|range| into_range(range));
            let right_answer = right.map(|range| into_range(range));

            let (left, right) = get_left_and_right(&super_range, &sub_range);

            assert_eq!(left, left_answer);
            assert_eq!(right, right_answer);
        }
    }

    #[test]
    fn split_test() {
        for (
            (a_range, a_val),
            (b_range, b_val),
            a_left,
            a_right,
            overlap,
            b_left,
            b_right,
        ) in [
            // [0, 70] vs [30, 100]
            // ->
            // [0, 30), [30, 70], (70, 100]
            (
                ((Some(0), true, Some(70), true), 1i32),
                ((Some(30), true, Some(100), true), 2),
                Some((Some(0), true, Some(30), false)),
                None,
                Some((Some(30), true, Some(70), true)),
                None,
                Some((Some(70), false, Some(100), true)),
            ),
            // [0, 100] vs [20, 80)
            // ->
            // [0, 20), [20, 80), [80, 100]
            (
                ((Some(0), true, Some(100), true), 1),
                ((Some(20), true, Some(80), false), 2),
                Some((Some(0), true, Some(20), false)),
                Some((Some(80), true, Some(100), true)),
                Some((Some(20), true, Some(80), false)),
                None,
                None,
            ),
            // [0, 50] vs [50, 100]
            // ->
            // [0, 50), [50, 50], (50, 100]
            (
                ((Some(0), true, Some(50), true), 1),
                ((Some(50), true, Some(100), true), 2),
                Some((Some(0), true, Some(50), false)),
                None,
                Some((Some(50), true, Some(50), true)),
                None,
                Some((Some(50), false, Some(100), true)),
            ),
        ] {
            let a_range = into_range(a_range);
            let b_range = into_range(b_range);

            let a_left_answer = a_left.map(|range| into_range(range));
            let a_right_answer = a_right.map(|range| into_range(range));
            let b_left_answer = b_left.map(|range| into_range(range));
            let b_right_answer = b_right.map(|range| into_range(range));
            let overlap_answer = overlap.map(|range| into_range(range));

            let result = split_to_non_overlapping_ranges(&(a_range, a_val), &(b_range, b_val));

            if let Some((a_left, a_val_v)) = &result.a_left {
                assert_eq!(*a_val_v, a_val);
                assert_eq!(a_left, &a_left_answer.unwrap());
            }

            else {
                assert!(a_left_answer.is_none());
            }

            if let Some((a_right, a_val_v)) = &result.a_right {
                assert_eq!(*a_val_v, a_val);
                assert_eq!(a_right, &a_right_answer.unwrap());
            }

            else {
                assert!(a_right_answer.is_none());
            }

            if let Some((b_left, b_val_v)) = &result.b_left {
                assert_eq!(*b_val_v, b_val);
                assert_eq!(b_left, &b_left_answer.unwrap());
            }

            else {
                assert!(b_left_answer.is_none());
            }

            if let Some((b_right, b_val_v)) = &result.b_right {
                assert_eq!(*b_val_v, b_val);
                assert_eq!(b_right, &b_right_answer.unwrap());
            }

            else {
                assert!(b_right_answer.is_none());
            }

            if let Some((overlap_range, overlap_val)) = &result.overlap {
                assert_eq!(*overlap_val, a_val.merge(&b_val));
                assert_eq!(overlap_range, &overlap_answer.unwrap());
            }

            else {
                assert!(overlap_answer.is_none());
            }
        }
    }
}

fn are_arms_eq<T, U, V, W>(arms1: &(Vec<(usize, T)>, U), arms2: &(Vec<(usize, V)>, W)) -> bool {
    arms1.0.len() == arms2.0.len() &&
    arms1.0.iter().zip(arms2.0.iter()).all(
        |((id1, _), (id2, _))| *id1 == *id2
    )
}
