mod error;
mod impls;
mod indented_lines;

#[cfg(test)]
mod tests;

pub use error::DecodeError;
pub use indented_lines::IndentedLines;

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
