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
        let i = a.len().min(32);
        let j = b.len().min(32);
        let mut cache = vec![vec![usize::MAX; j]; i];

        edit_distance_impl(
            if a.len() > 32 { &a[..32] } else { a },
            if b.len() > 32 { &b[..32] } else { b },
            i - 1,
            j - 1,
            &mut cache,
        )
    }
}

// lowercase
// remove `_`s
fn preprocess(s: &[u8]) -> Vec<u8> {
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
