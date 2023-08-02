#[inline]
pub fn get_prelude_buffs_len() -> usize {
    8
}

// Some context has to figure out whether an `InternedString` is underbar or not without session
pub const UNDERBAR_INDEX: usize = 1;

pub fn get_prelude_buffs() -> Vec<Vec<u8>> {
    vec![
        b"_".to_vec(),
        b"Int".to_vec(),
        b"String".to_vec(),
        b"List".to_vec(),
        b"Bool".to_vec(),
        b"Option".to_vec(),
        b"Func".to_vec(),
        b"test".to_vec(),
    ]
}

#[test]
fn prelude_buffs() {
    assert_eq!(get_prelude_buffs().len(), get_prelude_buffs_len());
}

#[test]
fn underbar_index() {
    let index = 0;
    assert_eq!(get_prelude_index(index), UNDERBAR_INDEX);
    assert_eq!(get_prelude_buffs()[index], b"_");
}

// get_prelude_index(i) == j, where
// InternedString(j) == get_prelude_buffs()[i]
pub fn get_prelude_index(i: usize) -> usize {
    i + 1
}
