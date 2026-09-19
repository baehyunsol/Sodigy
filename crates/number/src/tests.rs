use crate::{
    InternedNumber,
    BigInt,
    // Ratio,
    add_bi,
    add_ubi,
    div_bi,
    div_ubi,
    mul_bi,
    mul_ubi,
    rem_bi,
    rem_ubi,
    shl_ubi,
    shr_ubi,
    sub_bi,
    sub_ubi,
};
use std::mem::size_of;

#[test]
fn size_assertions() {
    assert!(size_of::<InternedNumber>() <= 16, "{}", size_of::<InternedNumber>());

    // It's okay for these to be big! That's why `InternedNumber` exists...
    // assert!(size_of::<BigInt>() <= 48, "{}", size_of::<BigInt>());
    // assert!(size_of::<Ratio>() <= 48, "{}", size_of::<Ratio>());
}

#[test]
fn interned_small_integer() {
    let n = InternedNumber::from_u32(30, true);
    assert_eq!(i32::try_from(n).unwrap() as u32, 30);
    assert_eq!(i64::try_from(n).unwrap() as u32, 30);
    assert_eq!(u32::try_from(n).unwrap(), 30);
    assert_eq!(u64::try_from(n).unwrap() as u32, 30);

    let n = InternedNumber::from_u32(0, true);
    assert_eq!(i32::try_from(n).unwrap() as u32, 0);
    assert_eq!(i64::try_from(n).unwrap() as u32, 0);
    assert_eq!(u32::try_from(n).unwrap(), 0);
    assert_eq!(u64::try_from(n).unwrap() as u32, 0);

    let n = InternedNumber::from_i32(-30, true);
    assert_eq!(i32::try_from(n).unwrap(), -30);
    assert_eq!(i64::try_from(n).unwrap() as i32, -30);

    let n = InternedNumber::from_i32(0, true);
    assert_eq!(i32::try_from(n).unwrap(), 0);
    assert_eq!(i64::try_from(n).unwrap() as i32, 0);
    assert_eq!(u32::try_from(n).unwrap() as i32, 0);
    assert_eq!(u64::try_from(n).unwrap() as i32, 0);
}

#[test]
fn u128_bi_arith_test() {
    let sample = vec![
        vec![0],
        vec![1],
        vec![2],
        vec![100_000],
        vec![200_000],
        vec![300_000],
        vec![100_000, 100_000],
        vec![100_000, 200_000],
        vec![100_000, 300_000],
        vec![200_000, 100_000],
        vec![200_000, 200_000],
        vec![200_000, 300_000],
        vec![300_000, 100_000],
        vec![300_000, 200_000],
        vec![300_000, 300_000],
        vec![5, 5],
        vec![5, 10],
        vec![10, 5],
        vec![0, 0, 1],
        vec![0, 0, 2],
        vec![0, 0, 3],
        vec![128, 0, 1],
        vec![0, 128, 1],
        vec![128, 128, 1],
        vec![192, 0, 1],
        vec![0, 192, 1],
        vec![192, 192, 1],
        vec![128, 0, 2],
        vec![0, 128, 2],
        vec![128, 128, 2],
        vec![192, 0, 2],
        vec![0, 192, 2],
        vec![192, 192, 2],
        vec![128, 0, 3],
        vec![0, 128, 3],
        vec![128, 128, 3],
        vec![192, 0, 3],
        vec![0, 192, 3],
        vec![192, 192, 3],
        vec![1, 3, 5, 7],
        vec![3, 5, 7, 1],
        vec![5, 7, 1, 3],
        vec![7, 1, 3, 5],
        vec![7, 5, 3, 1],
        vec![7, u32::MAX, 5, 5],
    ];

    let mut signed_sample = vec![];

    for nums in sample.iter() {
        if &nums[..] == &[0] {
            signed_sample.push((false, nums.to_vec()));
        } else {
            signed_sample.push((true, nums.to_vec()));
            signed_sample.push((false, nums.to_vec()));
        }
    }

    for (is_neg_a, nums_a) in signed_sample.iter() {
        let bi_a = BigInt { is_neg: *is_neg_a, nums: nums_a.to_vec() };
        let i128_a = i128::try_from(&bi_a).unwrap();

        for (is_neg_b, nums_b) in signed_sample.iter() {
            let bi_b = BigInt { is_neg: *is_neg_b, nums: nums_b.to_vec() };
            let i128_b = i128::try_from(&bi_b).unwrap();
            println!("a: {i128_a}, b: {i128_b}");

            let (is_neg_c, nums_c) = add_bi(*is_neg_a, nums_a, *is_neg_b, nums_b);
            let bi_c = BigInt { is_neg: is_neg_c, nums: nums_c.clone() };
            let i128_c = i128::try_from(&bi_c).unwrap();
            assert_eq!(i128_c, i128_a + i128_b);

            if !*is_neg_a && !*is_neg_b {
                assert_eq!(nums_c, add_ubi(nums_a, nums_b));
            }

            let (is_neg_d, nums_d) = sub_bi(*is_neg_a, nums_a, *is_neg_b, nums_b);
            let bi_d = BigInt { is_neg: is_neg_d, nums: nums_d.clone() };
            let i128_d = i128::try_from(&bi_d).unwrap();
            assert_eq!(i128_d, i128_a - i128_b);

            if !*is_neg_a && !*is_neg_b {
                assert_eq!(nums_d, sub_ubi(nums_a, nums_b));
            }

            if let Some(answer) = i128_a.checked_mul(i128_b) {
                let (is_neg_e, nums_e) = mul_bi(*is_neg_a, nums_a, *is_neg_b, nums_b);
                let bi_e = BigInt { is_neg: is_neg_e, nums: nums_e.clone() };
                let i128_e = i128::try_from(&bi_e).unwrap();
                assert_eq!(i128_e, answer);

                if !*is_neg_a && !*is_neg_b {
                    assert_eq!(nums_e, mul_ubi(nums_a, nums_b));
                }
            }

            if let Some(answer) = i128_a.checked_div(i128_b) {
                let (is_neg_f, nums_f) = div_bi(*is_neg_a, nums_a, *is_neg_b, nums_b);
                let bi_f = BigInt { is_neg: is_neg_f, nums: nums_f.clone() };
                let i128_f = i128::try_from(&bi_f).unwrap();
                assert_eq!(i128_f, answer);

                if !*is_neg_a && !*is_neg_b {
                    assert_eq!(nums_f, div_ubi(nums_a, nums_b));
                }
            }

            if let Some(answer) = i128_a.checked_rem(i128_b) {
                let (is_neg_g, nums_g) = rem_bi(*is_neg_a, nums_a, *is_neg_b, nums_b);
                let bi_g = BigInt { is_neg: is_neg_g, nums: nums_g.clone() };
                let i128_g = i128::try_from(&bi_g).unwrap();
                assert_eq!(i128_g, answer);

                if !*is_neg_a && !*is_neg_b {
                    assert_eq!(nums_g, rem_ubi(nums_a, nums_b));
                }
            }
        }

        let u128_a = u128::try_from(&BigInt { is_neg: false, nums: nums_a.to_vec() }).unwrap();

        for shift in [
            0, 1, 2, 3, 4,
            14, 15, 16, 17, 18,
            30, 31, 32, 33, 34,
            62, 63, 64, 65, 66,
            94, 95, 96, 97, 98,
            126, 127, 128,
        ] {
            if let Some(answer) = u128_a.checked_shl(shift) {
                let nums_h = shl_ubi(nums_a, shift);
                let bi_h = BigInt { is_neg: false, nums: nums_h };

                if let Ok(u128_h) = u128::try_from(&bi_h) {
                    assert_eq!(u128_h, answer);
                }
            }

            if let Some(answer) = u128_a.checked_shr(shift) {
                let nums_i = shr_ubi(nums_a, shift);
                let bi_i = BigInt { is_neg: false, nums: nums_i };
                let u128_i = u128::try_from(&bi_i).unwrap();
                assert_eq!(u128_i, answer);
            }
        }

        for long_shift in [
            8, 16, 24, 32,
            40, 48, 56, 64,
            72, 80, 88, 96,
            104, 112, 120, 128,
            136, 144, 152, 160,
            168, 176, 184, 192,
            200, 208, 216, 224,
            232, 240, 248, 256,
            264, 272, 280, 288,
            296, 304, 312, 320,
            328, 336, 344, 352,
        ] {
            let shifted = shl_ubi(nums_a, long_shift);
            let shifted_back = shr_ubi(&shifted, long_shift);
            assert_eq!(nums_a, &shifted_back);
        }
    }
}
