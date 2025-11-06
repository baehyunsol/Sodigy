use crate::{File, ModulePath};
use sodigy_endec::{DecodeError, Endec};

impl Endec for File {
    fn encode_impl(&self, buffer: &mut Vec<u8>) {
        match self {
            File::File { project, file } => {
                buffer.push(0);
                project.encode_impl(buffer);
                file.encode_impl(buffer);
            },
            File::Std(n) => {
                buffer.push(1);
                n.encode_impl(buffer);
            },
        }
    }

    fn decode_impl(buffer: &[u8], cursor: usize) -> Result<(Self, usize), DecodeError> {
        match buffer.get(cursor) {
            Some(0) => {
                let (project, cursor) = u32::decode_impl(buffer, cursor + 1)?;
                let (file, cursor) = u32::decode_impl(buffer, cursor)?;
                Ok((File::File { project, file }, cursor))
            },
            Some(1) => {
                let (n, cursor) = u64::decode_impl(buffer, cursor + 1)?;
                Ok((File::Std(n), cursor))
            },
            Some(n @ 2..) => Err(DecodeError::InvalidEnumVariant(*n)),
            None => Err(DecodeError::UnexpectedEof),
        }
    }
}

impl Endec for ModulePath {
    fn encode_impl(&self, buffer: &mut Vec<u8>) {
        self.path.encode_impl(buffer);
        self.is_std.encode_impl(buffer);
    }

    fn decode_impl(buffer: &[u8], cursor: usize) -> Result<(Self, usize), DecodeError> {
        let (path, cursor) = Vec::<String>::decode_impl(buffer, cursor)?;
        let (is_std, cursor) = bool::decode_impl(buffer, cursor)?;

        Ok((ModulePath { path, is_std }, cursor))
    }
}
