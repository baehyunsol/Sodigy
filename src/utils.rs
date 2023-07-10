pub fn into_char(s: &[u8], ind: usize) -> char {
    if s[ind] < 128 {
        s[ind] as char
    } else if s[ind] < 224 {
        std::str::from_utf8(&s[ind..(ind + 2)])
            .expect(&format!("Internal Compiler Error 5907096: {:?}", &s[ind..(ind + 2)]))
            .chars()
            .collect::<Vec<char>>()[0]
    } else if s[ind] < 240 {
        std::str::from_utf8(&s[ind..(ind + 3)])
            .expect(&format!("Internal Compiler Error 0371E1F: {:?}", &s[ind..(ind + 3)]))
            .chars()
            .collect::<Vec<char>>()[0]
    } else if s[ind] < 248 {
        std::str::from_utf8(&s[ind..(ind + 4)])
            .expect(&format!("Internal Compiler Error B862683: {:?}", &s[ind..(ind + 4)]))
            .chars()
            .collect::<Vec<char>>()[0]
    } else {
        unreachable!("Internal Compiler Error 9684A25: {s:?}, {ind}")
    }
}

// every Vec<u8> in the compiler must be a valid UTF-8,
// invalid UTF-8 must be rejected beforehand
pub fn bytes_to_string(b: &[u8]) -> String {
    String::from_utf8(b.to_vec()).expect("Internal Compiler Error 0A502DB: {b:?}")
}

// https://en.wikipedia.org/wiki/Damerau%E2%80%93Levenshtein_distance
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

#[test]
fn into_char_test() {
    let s = "aͲ린".as_bytes();

    assert_eq!(into_char(s, 0), 'a');
    assert_eq!(into_char(s, 1), 'Ͳ');
    assert_eq!(into_char(s, 3), '린');
}

#[test]
fn edit_distance_test() {
    assert_eq!(edit_distance(b"item", b"itme"), 1);
    assert_eq!(edit_distance(b"time", b"tiem"), 1);
    assert_eq!(edit_distance(b"Internal", b"Interal"), 1);
    assert_eq!(edit_distance(b"HTML", b"Programming Language"), 18);

    assert_eq!(substr_edit_distance(b"edit_distan", b"substr_edit_distance"), 0);
    assert_eq!(substr_edit_distance(b"edit_dustan", b"substr_edit_distance"), 1);
}