use super::SdgHash;

#[test]
fn str_hash() {
    assert_eq!(
        "".sdg_hash().to_bytes(),
        b"e3b0c44298fc1c149afbf4c8996fb92427ae41e4649b934ca495991b7852b855",
    );
    assert_eq!(
        "sodigy".sdg_hash().to_bytes(),
        b"b608af168113c808f5ce71ebca4982e0bee2811c3db5e08fcbbef3800dce03a7",
    );
}

#[test]
fn int_hash() {
    for i in 0..64 {
        assert_eq!(
            (i as u32).sdg_hash(),
            (i as u64).sdg_hash(),
        );
    }
}
