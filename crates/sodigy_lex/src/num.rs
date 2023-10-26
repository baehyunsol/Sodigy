pub mod err;

use super::err::ExpectedChars;
use err::ParseNumberError;

pub fn bin_to_dec(n: &[u8]) -> Result<Vec<u8>, ParseNumberError> {
    if n.is_empty() {
        return Err(ParseNumberError::UnfinishedNumLiteral(ExpectedChars::Specific(b"01_".to_vec())));
    }

    let mut result = vec![b'0'; 8];

    for c in n.iter() {
        mul_n::<2>(&mut result);
        let result_len = result.len();

        result[result_len - 1] += c - b'0';
    }

    // carry carries
    mul_n::<1>(&mut result);

    let mut leading_zeros = 0;

    while leading_zeros + 1 < result.len() && result[leading_zeros] == b'0' {
        leading_zeros += 1;
    }

    result = result[leading_zeros..].to_vec();

    Ok(result)
}

pub fn oct_to_dec(n: &[u8]) -> Result<Vec<u8>, ParseNumberError> {
    if n.is_empty() {
        return Err(ParseNumberError::UnfinishedNumLiteral(ExpectedChars::Specific(b"01234567_".to_vec())));
    }

    let mut result = vec![b'0'; 8];

    for c in n.iter() {
        mul_n::<8>(&mut result);
        let result_len = result.len();

        result[result_len - 1] += c - b'0';
    }

    // carry carries
    mul_n::<1>(&mut result);

    let mut leading_zeros = 0;

    while leading_zeros + 1 < result.len() && result[leading_zeros] == b'0' {
        leading_zeros += 1;
    }

    result = result[leading_zeros..].to_vec();

    Ok(result)
}

pub fn hex_to_dec(n: &[u8]) -> Result<Vec<u8>, ParseNumberError> {
    if n.is_empty() {
        return Err(ParseNumberError::UnfinishedNumLiteral(ExpectedChars::Specific(b"0123456789aAbBcCdDeEfF_".to_vec())));
    }

    let mut result = vec![b'0'; 8];

    for c in n.iter() {
        mul_n::<16>(&mut result);
        let result_len = result.len();

        result[result_len - 1] += to_n(*c);
    }

    // carry carries
    mul_n::<1>(&mut result);

    let mut leading_zeros = 0;

    while leading_zeros + 1 < result.len() && result[leading_zeros] == b'0' {
        leading_zeros += 1;
    }

    result = result[leading_zeros..].to_vec();

    Ok(result)
}

fn mul_n<const N: u8>(n: &mut Vec<u8>) {
    n.iter_mut().for_each(|n| { *n = (*n - b'0') * N; });
    let n_len = n.len();

    for i in 1..n_len {
        n[n_len - i - 1] += n[n_len - i] / 10;
        n[n_len - i] %= 10;
        n[n_len - i] += b'0';
    }

    n[0] += b'0';

    if n[0] > b'9' {
        *n = vec![vec![b'0'; 8], n.to_vec()].concat();
        n[7] = (n[8] - b'0') / 10 + b'0';
        n[8] = (n[8] - b'0') % 10 + b'0';
    }
}

fn to_n(c: u8) -> u8 {
    if c <= b'9' {
        c - b'0'
    }

    else if c <= b'Z' {
        c - b'A' + 10
    }

    else {
        c - b'a' + 10
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::err::ExpectedChars;

    #[test]
    fn xxx_to_dec() {
        assert_eq!(bin_to_dec(&vec![b'1', b'1', b'0', b'1']), Ok(vec![b'1', b'3']));
        assert_eq!(bin_to_dec(&vec![b'1'; 64]), Ok(u64::MAX.to_string().as_bytes().to_vec()));
        assert_eq!(oct_to_dec(&vec![b'1', b'1', b'0', b'1']), Ok(vec![b'5', b'7', b'7']));
        assert_eq!(oct_to_dec(&vec![b'7'; 16]), Ok(((1u64 << 48) - 1).to_string().as_bytes().to_vec()));
        assert_eq!(hex_to_dec(&vec![b'1', b'1', b'0', b'1']), Ok(vec![b'4', b'3', b'5', b'3']));
        assert_eq!(hex_to_dec(&vec![b'f'; 16]), Ok(u64::MAX.to_string().as_bytes().to_vec()));
        assert_eq!(hex_to_dec(&vec![]), Err(ParseNumberError::UnfinishedNumLiteral(ExpectedChars::Specific(b"0123456789aAbBcCdDeEfF_".to_vec()))));
    }
}
