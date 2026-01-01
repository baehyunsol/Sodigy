use crate::{intern_string, unintern_string};
use sodigy_fs_api::{create_dir_all, exists, remove_dir_all};

#[test]
fn tests() {
    if exists("data") {
        remove_dir_all("data").unwrap();
    }

    create_dir_all("data/str").unwrap();

    let mut strings: Vec<&[u8]> = vec![
        b"",
        b"0", b"1", b"2",
        b"12345", b"abcde",
        b"0123456789101112131415",
        b"01234567891011121314151617",
        b"abcdefghijklmnopqrstuv",
        b"abcdefghijklmnopqrstuvwxyz",
        b"This is a very very long string. I need a string that is at least 256 bytes long, because my code treats long strings differently. I'm worried if it's some kinda over-engineered. But it's fun though, and I'm not gonna over-engineering my whole code base. I just need a few more characters to make it longer than 256 bytes.",
    ];

    // We need strings that are longer than 24MiB.
    let very_long_string1 = b"abcde".repeat(3355444);
    let very_long_string2 = b"asdfg".repeat(3355444);
    let very_long_string3 = b"jklmn".repeat(3355444);
    strings.push(&very_long_string1);
    strings.push(&very_long_string2);
    strings.push(&very_long_string3);

    let interned_strings = strings.iter().map(
        |s| match intern_string(s, "data") {
            Ok(s) => s,
            Err(e) => panic!("Failed to intern_string({s:?}): {e:?}"),
        }
    ).collect::<Vec<_>>();
    let uninterned_interned_strings = interned_strings.iter().enumerate().map(
        |(i, s)| match unintern_string(*s, "data") {
            Ok(Some(s)) => s,
            Ok(None) => panic!("unintern_string({s:?}): data not found (supposed to be {:?})", strings[i]),
            Err(e) => panic!("Failed to unintern_string({s:?}): {e:?} (supposed to be {:?})", strings[i]),
        }
    ).collect::<Vec<_>>();

    for (i, (s, (is, uis))) in strings.iter().zip(interned_strings.iter().zip(uninterned_interned_strings.iter())).enumerate() {
        let (l1, l2, l3) = (s.len(), is.len(), uis.len());
        assert_eq!(l1, l2);
        assert_eq!(l2, l3);

        assert_eq!(s, uis);
        assert!(is.eq(s));
        assert!(is.eq(uis));

        for j in (i + 1)..strings.len() {
            let (another_s, another_uis) = (strings[j], &uninterned_interned_strings[j]);
            assert!(!is.eq(another_s));
            assert!(!is.eq(another_uis));
        }
    }

    remove_dir_all("data").unwrap();
}
