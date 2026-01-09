use crate::{Ratio, cmp_bi, mul_bi};
use std::cmp::Ordering;

pub fn cmp_ratio(lhs: &Ratio, rhs: &Ratio) -> Ordering {
    let (lhs_is_neg, lhs_nums) = mul_bi(lhs.numer.is_neg, &lhs.numer.nums, rhs.denom.is_neg, &rhs.denom.nums);
    let (rhs_is_neg, rhs_nums) = mul_bi(rhs.numer.is_neg, &rhs.numer.nums, lhs.denom.is_neg, &lhs.denom.nums);
    cmp_bi(lhs_is_neg, &lhs_nums, rhs_is_neg, &rhs_nums)
}
