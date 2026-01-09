use crate::{
    BigInt,
    Ratio,
    add_bi,
    div_bi,
    gcd_ubi,
    mul_bi,
    sub_bi,
};

pub fn add_ratio(lhs: &Ratio, rhs: &Ratio) -> Ratio {
    let lhs_numer = mul_bi(lhs.numer.is_neg, &lhs.numer.nums, rhs.denom.is_neg, &rhs.denom.nums);
    let rhs_numer = mul_bi(rhs.numer.is_neg, &rhs.numer.nums, lhs.denom.is_neg, &lhs.denom.nums);

    let res_numer = add_bi(lhs_numer.0, &lhs_numer.1, rhs_numer.0, &rhs_numer.1);
    let res_denom = mul_bi(lhs.denom.is_neg, &lhs.denom.nums, rhs.denom.is_neg, &rhs.denom.nums);
    reduce_and_return(res_numer, res_denom)
}

pub fn sub_ratio(lhs: &Ratio, rhs: &Ratio) -> Ratio {
    let lhs_numer = mul_bi(lhs.numer.is_neg, &lhs.numer.nums, rhs.denom.is_neg, &rhs.denom.nums);
    let rhs_numer = mul_bi(rhs.numer.is_neg, &rhs.numer.nums, lhs.denom.is_neg, &lhs.denom.nums);

    let res_numer = sub_bi(lhs_numer.0, &lhs_numer.1, rhs_numer.0, &rhs_numer.1);
    let res_denom = mul_bi(lhs.denom.is_neg, &lhs.denom.nums, rhs.denom.is_neg, &rhs.denom.nums);
    reduce_and_return(res_numer, res_denom)
}

pub fn mul_ratio(lhs: &Ratio, rhs: &Ratio) -> Ratio {
    let res_numer = mul_bi(lhs.numer.is_neg, &lhs.numer.nums, rhs.numer.is_neg, &rhs.numer.nums);
    let res_denom = mul_bi(lhs.denom.is_neg, &lhs.denom.nums, rhs.denom.is_neg, &rhs.denom.nums);
    reduce_and_return(res_numer, res_denom)
}

pub fn div_ratio(lhs: &Ratio, rhs: &Ratio) -> Ratio {
    let res_numer = mul_bi(lhs.numer.is_neg, &lhs.numer.nums, rhs.denom.is_neg, &rhs.denom.nums);
    let res_denom = mul_bi(lhs.denom.is_neg, &lhs.denom.nums, rhs.numer.is_neg, &rhs.numer.nums);
    reduce_and_return(res_numer, res_denom)
}

fn reduce_and_return(mut numer: (bool, Vec<u32>), mut denom: (bool, Vec<u32>)) -> Ratio {
    let r = gcd_ubi(&numer.1, &denom.1);

    if &r != &[1] {
        numer = div_bi(numer.0, &numer.1, false, &r);
        denom = div_bi(denom.0, &denom.1, false, &r);
    }

    Ratio {
        numer: BigInt {
            is_neg: numer.0,
            nums: numer.1,
        },
        denom: BigInt {
            is_neg: denom.0,
            nums: denom.1,
        },
    }
}
