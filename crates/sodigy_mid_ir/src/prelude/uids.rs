use sodigy_uid::Uid;

// TODO: how do we initialize these?
pub const INT_DEF: Uid = Uid::dummy(100).mark_prelude();
pub const RATIO_DEF: Uid = Uid::dummy(101).mark_prelude();
pub const RATIO_INIT: Uid = Uid::dummy(102).mark_prelude();
pub const CHAR_DEF: Uid = Uid::dummy(103).mark_prelude();
pub const LIST_DEF: Uid = Uid::dummy(104).mark_prelude();
pub const LIST_INIT: Uid = Uid::dummy(105).mark_prelude();
pub const STRING_DEF: Uid = Uid::dummy(106).mark_prelude();
pub const BYTES_DEF: Uid = Uid::dummy(107).mark_prelude();
pub const BYTE_DEF: Uid = Uid::dummy(108).mark_prelude();

// `Func` type
pub const FUNC_DEF: Uid = Uid::dummy(109).mark_prelude();
pub const BOOL_DEF: Uid = Uid::dummy(110).mark_prelude();
pub const BOOL_VARIANT_TRUE: Uid = Uid::dummy(111).mark_prelude();
pub const BOOL_VARIANT_FALSE: Uid = Uid::dummy(112).mark_prelude();
