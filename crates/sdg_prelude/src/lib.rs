#[inline]
pub fn get_prelude_buffs_len() -> usize {
    6
}

pub fn get_prelude_buffs() -> Vec<Vec<u8>> {
    vec![
        b"Int".to_vec(),
        b"String".to_vec(),
        b"List".to_vec(),
        b"Bool".to_vec(),
        b"Func".to_vec(),
        b"test".to_vec(),
    ]
}

#[test]
fn prelude_buffs() {
    assert_eq!(get_prelude_buffs().len(), get_prelude_buffs_len());
}

// get_prelude_index(i) == j, where
// InternedString(j) == get_prelude_buffs()[i]
pub fn get_prelude_index(i: usize) -> usize {
    i + 1
}
