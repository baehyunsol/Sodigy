mod error;
mod impls;

pub use error::DecodeError;

pub trait Endec {
    fn encode(&self) -> Vec<u8> {
        let mut result = vec![];
        self.encode_impl(&mut result);
        result
    }

    fn decode(buffer: &[u8]) -> Result<Self, DecodeError> where Self: Sized {
        let (result, cursor) = Self::decode_impl(buffer, 0)?;

        if cursor == buffer.len() {
            Ok(result)
        }

        else {
            Err(DecodeError::RemainingBytes)
        }
    }

    fn encode_impl(&self, buffer: &mut Vec<u8>);
    fn decode_impl(buffer: &[u8], cursor: usize) -> Result<(Self, usize), DecodeError> where Self: Sized;
}

// It dumps contents of a session in a human-readable format.
pub trait DumpIr {
    fn dump_ir(&self) -> Vec<u8>;
}

// `DumpIr` is used to dump sessions.
// But the last stage (code gen) doesn't have a session and instead directly generates the code (in Vec<u8>).
// So, this trait is used to dump the code (which is nop).
impl DumpIr for Vec<u8> {
    fn dump_ir(&self) -> Vec<u8> {
        self.to_vec()
    }
}
