pub fn into_char(s: &[u8], ind: usize) -> char {

    if s[ind] < 128 {
        s[ind] as char
    }

    else if s[ind] < 224 {
        std::str::from_utf8(&s[ind..(ind + 2)]).unwrap_or("?").chars().collect::<Vec<char>>()[0]
    }

    else if s[ind] < 240 {
        std::str::from_utf8(&s[ind..(ind + 3)]).unwrap_or("?").chars().collect::<Vec<char>>()[0]
    }

    else if s[ind] < 248 {
        std::str::from_utf8(&s[ind..(ind + 4)]).unwrap_or("?").chars().collect::<Vec<char>>()[0]
    }

    else {
        unreachable!("Internal Compiler Error 9684A25: {s:?}, {ind}")
    }

}

#[test]
fn into_char_test() {
    let s = "aͲ린".as_bytes();

    assert_eq!(into_char(s, 0), 'a');
    assert_eq!(into_char(s, 1), 'Ͳ');
    assert_eq!(into_char(s, 3), '린');
}