use crate::{Endec, EndecSession};

macro_rules! vec_int_roundtrip {
    ($t: ty, $m: ident) => {
        #[test]
        fn $m() {
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

            let mut sess = EndecSession::new();

            for sample in samples.into_iter() {
                let mut buffer = vec![];

                sample.encode(&mut buffer, &mut sess);
                let buffer_c = buffer.clone();
                let t = Vec::<$t>::decode(&buffer, &mut 0, &mut sess).unwrap();

                let mut buffer = vec![];
                t.encode(&mut buffer, &mut sess);

                assert_eq!(sample, t);
                assert_eq!(buffer, buffer_c);
            }
        }
    }
}

vec_int_roundtrip!(u64, vec_u64_roundtrip);
vec_int_roundtrip!(i64, vec_i64_roundtrip);
vec_int_roundtrip!(u128, vec_u128_roundtrip);
vec_int_roundtrip!(i128, vec_i128_roundtrip);
