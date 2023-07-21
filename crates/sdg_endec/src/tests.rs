use super::Endec;

#[test]
fn vec_int_test() {
    vec_int(vec![
        1u64,
        11, 111, 1_111,
        11_111, 111_111,
        1_111_111,
        11_111_111,
        111_111_111,
        1_111_111_111,
        11_111_111_111,
        111_111_111_111,
        1_111_111_111_111,
    ]);

    vec_int(vec![
        -1, 0, 1i64,
        -9999, 0, 9999,
        1307674368000,
        -1307674368000,
        i64::MIN, i64::MAX,
    ]);
}

fn vec_int<T: Endec + std::fmt::Debug + PartialEq>(v: Vec<T>) {
    let mut buf = vec![];

    v.encode(&mut buf);

    let mut index = 0;
    let v2 = Vec::<T>::decode(&buf, &mut index).unwrap();

    assert_eq!(v, v2);
}
