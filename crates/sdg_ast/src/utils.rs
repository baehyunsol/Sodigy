use crate::err::ParseError;

pub fn print_list(list: &[String], wrapper_pre: &str, wrapper_post: &str, and_or_or: &str) -> String {

    if list.is_empty() {
        String::new()
    }

    else if list.len() == 1 {
        format!("{wrapper_pre}{}{wrapper_post}", list[0])
    }

    else if list.len() == 2 {
        format!("{wrapper_pre}{}{wrapper_post} {and_or_or} {wrapper_pre}{}{wrapper_post}", list[0], list[1])
    }

    else {
        format!("{wrapper_pre}{}{wrapper_post}, {}", list[0], print_list(&list[1..], wrapper_pre, wrapper_post, and_or_or))
    }

}

pub fn into_char(s: &[u8], ind: usize) -> Result<char, ParseError> {
    let len = if s[ind] < 128 {
        1
    } else if s[ind] < 224 {
        2
    } else if s[ind] < 240 {
        3
    } else if s[ind] < 248 {
        4
    } else {
        return Err(ParseError::invalid_utf8_dummy_index(vec![s[ind]], 0));
    };

    if let Ok(s) = std::str::from_utf8(&s[ind..(ind + len)]) {
        Ok(s.chars().nth(0).expect("Internal Compiler Error 039A331EDA3"))
    }

    else {
        Err(ParseError::invalid_utf8_dummy_index(s[ind..(ind + len)].to_vec(), 0))
    }

}

fn assemble_char(cs: Vec<u8>, ind: usize) -> Result<u32, ParseError> {
    assert!(cs.len() > 0, "Internal Compiler Error F5AB2B57A5E");

    if cs[0] < 192 {
        Err(ParseError::invalid_utf8_dummy_index(cs, ind))
    }

    else if cs[0] < 224 {

        if cs.len() != 2 {
            Err(ParseError::invalid_utf8_dummy_index(cs, ind))
        }

        else {
            Ok(cs[0] as u32 % 32 * 64 + cs[1] as u32 % 64)
        }

    }

    else if cs[0] < 240 {

        if cs.len() != 3 {
            Err(ParseError::invalid_utf8_dummy_index(cs, ind))
        }

        else {
            Ok(
                cs[0] as u32 % 16 * 4096
                + cs[1] as u32 % 64 * 64
                + cs[2] as u32 % 64
            )
        }

    }

    else if cs[0] < 248 {

        if cs.len() != 4 {
            Err(ParseError::invalid_utf8_dummy_index(cs, ind))
        }

        else {
            Ok(
                cs[0] as u32 % 8 * 262144
                + cs[1] as u32 % 64 * 4096
                + cs[2] as u32 % 64 * 64
                + cs[3] as u32 % 64
            )
        }

    }

    else {
        Err(ParseError::invalid_utf8_dummy_index(cs, ind))
    }

}

pub fn bytes_to_v32(s: &[u8]) -> Result<Vec<u32>, ParseError> {
    let mut result = Vec::with_capacity(s.len());
    let mut tmp_buf = vec![];

    for (ind, c) in s.iter().enumerate() {

        if tmp_buf.is_empty() {

            if *c < 128 {
                result.push(*c as u32);
            }

            else {
                tmp_buf.push(*c);
            }

        }

        else {

            if *c < 128 {
                result.push(assemble_char(tmp_buf, ind)?);
                result.push(*c as u32);
                tmp_buf = vec![];
            }

            else if *c >= 192 {
                result.push(assemble_char(tmp_buf, ind)?);
                tmp_buf = vec![*c];
            }

            else {
                tmp_buf.push(*c);
            }

        }

    }

    if !tmp_buf.is_empty() {
        let ind = s.len() - tmp_buf.len();
        result.push(assemble_char(tmp_buf, ind)?);
    }

    Ok(result)
}

pub fn v32_to_string(v: &[u32]) -> Result<String, u32> {
    let mut chars = Vec::with_capacity(v.len());

    for c in v.iter() {

        match char::from_u32(*c) {
            Some(c) => { chars.push(c); }
            None => {
                return Err(*c)
            }
        }

    }

    Ok(chars.iter().collect())
}

pub fn v32_to_bytes(v: &[u32]) -> Vec<u8> {
    let mut buffer = Vec::with_capacity(v.len() * 3 / 2);

    for c in v.iter() {

        if *c < 128 {
            buffer.push(*c as u8);
        }

        else if *c < 4096 {
            buffer.push(192 + (*c / 64) as u8);
            buffer.push(128 + (*c % 64) as u8);
        }

        else if *c < 65536 {
            buffer.push(224 + (*c / 4096) as u8);
            buffer.push(128 + (*c % 4096 / 64) as u8);
            buffer.push(128 + (*c % 64) as u8);
        }

        else if *c < 2097152 {
            buffer.push(240 + (*c / 262144) as u8);
            buffer.push(128 + (*c % 262144 / 4096) as u8);
            buffer.push(128 + (*c % 4096 / 64) as u8);
            buffer.push(128 + (*c % 64) as u8);
        }

        // I know that UTF-8 ends at 0x10ffff... but, you know, no one knows the future
        else {
            buffer.push(248 + (*c / 16777216) as u8);
            buffer.push(128 + (*c % 16777216 / 262144) as u8);
            buffer.push(128 + (*c % 262144 / 4096) as u8);
            buffer.push(128 + (*c % 4096 / 64) as u8);
            buffer.push(128 + (*c % 64) as u8);
        }

    }

    buffer
}

pub fn bytes_to_string(b: &[u8]) -> String {
    String::from_utf8_lossy(b).to_string()
}

/// https://en.wikipedia.org/wiki/Damerau%E2%80%93Levenshtein_distance
pub fn edit_distance(a: &[u8], b: &[u8]) -> usize {

    if a.is_empty() {

        if b.is_empty() {
            0
        }

        else {
            b.len()
        }

    }

    else if b.is_empty() {
        a.len()
    }

    else {
        let i = a.len();
        let j = b.len();
        let mut cache = vec![vec![usize::MAX; j]; i];

        edit_distance_impl(a, b, i - 1, j - 1, &mut cache)
    }

}

pub fn substr_edit_distance(sub: &[u8], s: &[u8]) -> usize {

    if sub == s {
        0
    }

    else if sub.len() > s.len() || s.len() < 4 {
        edit_distance(sub, s)
    }

    else if sub.len() * 2 > s.len() {
        let mut result = usize::MAX;

        for start in 0..s.len() {
            for end in (start + 1)..s.len() {
                result = result.min(
                    edit_distance(sub, &s[start..end])
                );
            }
        }

        result
    }

    else {
        edit_distance(sub, s)
    }

}

pub fn edit_distance_impl(a: &[u8], b: &[u8], i: usize, j: usize, cache: &mut Vec<Vec<usize>>) -> usize {

    if i == 0 && j == 0 {
        return 0;
    }

    if cache[i][j] != usize::MAX {
        return cache[i][j];
    }

    let mut result = usize::MAX;

    if i > 0 {
        result = result.min(edit_distance_impl(a, b, i - 1, j, cache) + 1);
    }

    if j > 0 {
        result = result.min(edit_distance_impl(a, b, i, j - 1, cache) + 1);
    }

    let indicator = (a[i] != b[j]) as usize;

    if i > 0 && j > 0 {
        result = result.min(edit_distance_impl(a, b, i - 1, j - 1, cache) + indicator);
    }

    if i > 1 && j > 1 && a[i] == b[j - 1] && a[i - 1] == b[j] {
        result = result.min(edit_distance_impl(a, b, i - 2, j - 2, cache) + indicator);
    }

    cache[i][j] = result;
    result
}

// TODO: make sure that adjacent characters are not alphanumeric
#[cfg(test)]
pub fn assert_identifier(s: String) {
    let mut is_identifier = !s.is_empty();

    for (i, c) in s.chars().enumerate() {

        if i == 0 && !(
            c.is_ascii_alphabetic()
            || c == '_'
        ) {
            is_identifier = false;
        }

        else if i != 0 && !(
            c.is_ascii_alphanumeric()
            || c == '_'
        ) {
            is_identifier = false;
        }

        if !is_identifier {
            break;
        }
    }

    if !is_identifier {
        panic!("{s} is not an identifier");
    }
}

#[cfg(test)]
mod tests {
    use super::{
        edit_distance, into_char, substr_edit_distance,
        bytes_to_string, bytes_to_v32,
        v32_to_bytes, v32_to_string,
    };

    #[test]
    fn into_char_test() {
        let s = "aͲ린".as_bytes();

        assert_eq!(into_char(s, 0).unwrap_or('X'), 'a');
        assert_eq!(into_char(s, 1).unwrap_or('X'), 'Ͳ');
        assert_eq!(into_char(s, 3).unwrap_or('X'), '린');
    }

    #[test]
    fn edit_distance_test() {
        assert_eq!(edit_distance(b"item", b"itme"), 1);
        assert_eq!(edit_distance(b"time", b"tiem"), 1);
        assert_eq!(edit_distance(b"Internal", b"Interal"), 1);
        assert_eq!(edit_distance(b"HTML", b"Programming Language"), 18);

        assert_eq!(substr_edit_distance(b"edit_distan", b"substr_edit_distance"), 0);
        assert_eq!(substr_edit_distance(b"edit_dustan", b"substr_edit_distance"), 1);

        assert_eq!(edit_distance(
            b"Very Very Long String: I want to make sure that `edit_distance` is not an O(a^n) algorithm",
            b"Another very very long string... 0123456789abcdefghijklmnopqrstuvwxyzABCDEFGHIJKLMNOPQRSTUVWXYZ",
        ), 75);
    }

    #[test]
    fn str_conversion_test() {
        let s1 = String::from("aaaπa가a🦈a👨🏿‍🎓πaπππ가π🦈π👨🏿‍🎓가a가π가가가🦈가👨🏿‍🎓🦈a🦈π🦈가🦈🦈🦈👨🏿‍🎓👨🏿‍🎓a👨🏿‍🎓π👨🏿‍🎓가👨🏿‍🎓🦈👨🏿‍🎓👨🏿‍🎓");
        let b1 = s1.as_bytes().to_vec();

        let v1 = bytes_to_v32(&b1).unwrap_or(vec![]);
        let s2 = v32_to_string(&v1).unwrap();
        let s3 = bytes_to_string(&b1);
        let b2 = v32_to_bytes(&v1);

        let v2 = bytes_to_v32(&b2).unwrap_or(vec![]);
        let s4 = v32_to_string(&v2).unwrap();
        let s5 = bytes_to_string(&b2);
        let b3 = v32_to_bytes(&v2);

        assert_eq!(s1, s2);
        assert_eq!(s2, s3);
        assert_eq!(s3, s4);
        assert_eq!(s4, s5);

        assert_eq!(v1, v2);

        assert_eq!(b1, b2);
        assert_eq!(b2, b3);
    }
}