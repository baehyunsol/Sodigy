use crate::File;
use sodigy_endec::{DecodeError, Endec};

impl Endec for File {
    fn encode_impl(&self, buffer: &mut Vec<u8>) {
        self.project.encode_impl(buffer);
        self.file.encode_impl(buffer);
    }

    fn decode_impl(buffer: &[u8], cursor: usize) -> Result<(Self, usize), DecodeError> {
        let (project, cursor) = u32::decode_impl(buffer, cursor)?;
        let (file, cursor) = u32::decode_impl(buffer, cursor)?;
        Ok((
            File {
                project,
                file,
            },
            cursor,
        ))
    }
}
