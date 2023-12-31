// https://en.wikipedia.org/wiki/Damerau%E2%80%93Levenshtein_distance
fn edit_distance(a: &[u8], b: &[u8]) -> usize {
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

// lowercase
// remove `_`s
fn preprocess(s: &[u8]) -> Vec<u8> {
    let s = if s.len() > 64 {
        &s[..64]
    } else {
        s
    };

    s.iter().map(
        |c| {
            let mut c = *c;
            c.make_ascii_lowercase();

            c
        }
    ).filter(
        |c| *c != b'_'
    ).collect()
}

// VERY EXPENSIVE
pub fn substr_edit_distance(sub: &[u8], s: &[u8]) -> usize {
    let sub = &preprocess(sub);
    let s = &preprocess(s);

    if sub == s {
        0
    }

    // I found that `edit_distance` cannot handle this case, I should fix that later
    else if sub.len() == 1 && s.len() == 1 {
        return (sub != s) as usize;
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

fn edit_distance_impl(a: &[u8], b: &[u8], i: usize, j: usize, cache: &mut Vec<Vec<usize>>) -> usize {
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
fn dist_test() {
    assert_eq!(substr_edit_distance(b"x", b"X"), 0);
    assert_eq!(substr_edit_distance(b"x", b"y"), 1);
    assert_eq!(substr_edit_distance(b"x", b"x1"), 1);
    assert_eq!(substr_edit_distance(b"item", b"itme"), 1);
    assert_eq!(substr_edit_distance(b"time", b"tiem"), 1);
    assert_eq!(substr_edit_distance(b"time", b"sime"), 1);
    assert_eq!(substr_edit_distance(b"Internal", b"Interal"), 1);
    assert_eq!(substr_edit_distance(b"HTML", b"Programming Language"), 17);

    assert_eq!(substr_edit_distance(b"edit_distan", b"substr_edit_distance"), 0);
    assert_eq!(substr_edit_distance(b"edit_dustan", b"substr_edit_distance"), 1);

    assert!(substr_edit_distance(
        "Very Very Long String: I want to make sure that `edit_distance` is not an O(a^n) algorithm".repeat(256).as_bytes(),
        "Another very very long string... 0123456789abcdefghijklmnopqrstuvwxyzABCDEFGHIJKLMNOPQRSTUVWXYZ".repeat(256).as_bytes(),
    ) > 10 /* the result doesn't matter, i just want to make sure that this code terminates in reasonable time */ );
}
