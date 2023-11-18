use crate::Endec;

#[test]
fn vec_int_roundtrip() {
    let samples = vec![
        vec![],
        vec![0],
        vec![0, 1, 2],
        vec![
            3, 9, 27, 81, 243, 729,
            2187, 6561, 19683, 59049,
            177147, 531441, 1594323,
            4782969, 14348907,
            43046721, 129140163,
            387420489, 1162261467,
            3486784401, 10460353203,
            31381059609, 94143178827,
            282429536481, 847288609443,
            2541865828329,
        ],
    ];

    for sample in samples.into_iter() {
        let mut buffer = vec![];

        sample.encode(&mut buffer);
        let buf_c = buffer.clone();
        let t = Vec::<u64>::decode(&buffer, &mut 0).unwrap();

        let mut buffer = vec![];
        t.encode(&mut buffer);

        assert_eq!(sample, t);
        assert_eq!(buffer, buf_c);
    }
}
