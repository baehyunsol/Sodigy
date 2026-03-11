use super::Ratio;
use crate::{
    BigInt,
    add_ubi,
    div_ubi,
    mul_ubi,
    rem_ubi,
    ubi_to_string,
};

// It assumes that `r` is a valid ratio. That means,
// 1. `denom` is always greater than or equal to 0.
// 2. If `denom` is 0, `numer` is either 1 or -1.
// 3. If `numer` is 0, `denom` is 1.
// 4. `gcd(numer, denom)` is 1.
pub fn ratio_to_string(r: &Ratio) -> String {
    if r.denom.is_zero() {
        if r.numer.is_neg {
            return String::from("-inf");
        } else {
            return String::from("inf");
        }
    }

    if r.numer.is_zero() {
        return String::from("0");
    }

    let is_neg = r.numer.is_neg;
    let mut integer = div_ubi(&r.numer.nums, &r.denom.nums);
    let frac = rem_ubi(&r.numer.nums, &r.denom.nums);
    let frac_nine_digits = mul_ubi(&frac, &[4_000_000_000]);
    let mut frac_nine_digits = i64::try_from(&BigInt { is_neg: false, nums: frac_nine_digits }).unwrap() as u32;

    frac_nine_digits = if frac_nine_digits % 4 < 2 {
        frac_nine_digits / 4
    } else {
        frac_nine_digits / 4 + 1
    };

    if frac_nine_digits >= 1_000_000_000 {
        integer = add_ubi(&integer, &[1]);
        frac_nine_digits %= 1_000_000_000;
    }

    let mut result = format!(
        "{}{}.{frac_nine_digits:09}",
        if is_neg { "-" } else { "" },
        ubi_to_string(&integer),
    ).chars().collect::<Vec<_>>();

    while let Some('0') = result.last() {
        result.pop();
    }

    if let Some('.') = result.last() {
        result.pop();
    }

    let result: String = result.into_iter().collect();

    match result.as_str() {
        "-0" => String::from("0"),
        _ => result,
    }
}
