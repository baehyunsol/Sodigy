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
    // `cache` uses O(n * m) space and I want to prevent it from OOM.
    let s = if s.len() > 256 {
        &s[..256]
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

pub fn get_closest_string(
    candidates: &[String],
    input: &str,
) -> Option<String> {
    let b = input.as_bytes();
    let mut close_strings = vec![];

    for c in candidates.iter() {
        let dist = substr_edit_distance(b, c.as_bytes());

        // Different strings can have 0-distance, since it does a case-insensitive comparison
        if dist <= input.len().min(c.len()) / 3 {
            close_strings.push((c.to_string(), dist));

            if dist == 0 {
                break;
            }
        }
    }

    close_strings.sort_by_key(|(_, dist)| *dist);
    close_strings.get(0).map(|(s, _)| s.to_string())
}

// VERY EXPENSIVE
// Please make sure that either `sub` or `s` is a short string, like a name of a command or a flag.
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
            for end in (start + 1)..(s.len() + 1) {
                result = result.min(edit_distance(sub, &s[start..end]));
            }
        }

        result
    }

    else {
        edit_distance(sub, s)
    }
}

fn edit_distance_impl(a: &[u8], b: &[u8], i: usize, j: usize, cache: &mut Vec<Vec<usize>>) -> usize {
    let indicator = (a[i] != b[j]) as usize;

    if i == 0 && j == 0 {
        return indicator;
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
    assert_eq!(substr_edit_distance(b"item", b"itm"), 1);
    assert_eq!(substr_edit_distance(b"time", b"tiem"), 1);
    assert_eq!(substr_edit_distance(b"time", b"sime"), 1);
    assert_eq!(substr_edit_distance(b"Internal", b"Interal"), 1);
    assert_eq!(substr_edit_distance(b"Interal", b"Internal"), 1);
    assert_eq!(substr_edit_distance(b"qqqqq", b"qqqqq"), 0);
    assert_eq!(substr_edit_distance(b"qqqqq", b"cqqqq"), 1);
    assert_eq!(substr_edit_distance(b"cqqqq", b"qqqqq"), 1);
    assert_eq!(substr_edit_distance(b"query", b"qeury"), 1);
    assert_eq!(substr_edit_distance(b"interactive", b"intercative"), 1);
    assert_eq!(substr_edit_distance(b"HTML", b"Programming Language"), 18);

    assert_eq!(substr_edit_distance(b"edit_distan", b"substr_edit_distance"), 0);
    assert_eq!(substr_edit_distance(b"edit_dustan", b"substr_edit_distance"), 1);

    assert!(substr_edit_distance(
        "Very Very Long String: I want to make sure that `edit_distance` is not an O(a^n) algorithm".repeat(256).as_bytes(),
        "Another very very long string... 0123456789abcdefghijklmnopqrstuvwxyzABCDEFGHIJKLMNOPQRSTUVWXYZ".repeat(256).as_bytes(),
    ) > 10 /* the result doesn't matter, i just want to make sure that this code terminates in reasonable time, without causing OOM */ );
}
