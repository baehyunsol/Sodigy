#![deny(unused_imports)]
use sodigy_files::{FileError, WriteMode, read_bytes, remove_file, write_bytes};

mod error;
mod impls;
mod session;

#[cfg(test)]
mod tests;

pub use error::{EndecError, EndecErrorContext, EndecErrorKind};
pub use session::EndecSession;

pub trait Endec {
    fn encode(&self, buf: &mut Vec<u8>, sess: &mut EndecSession);

    /// It moves the cursor (`ind`) after decoding. If the decoding fails, it may or may not move the cursor.
    fn decode(buf: &[u8], index: &mut usize, sess: &mut EndecSession) -> Result<Self, EndecError> where Self: Sized;

    // `path` and `file_metadata` point to different files
    // `path` is an ir file, and `file_metadata` is of `.sdg` file.
    fn save_to_file(&self, path: &str, file_metadata: Option<u64>) -> Result<(), FileError> {
        let mut buffer = vec![];
        let mut endec_session = EndecSession::new();

        self.encode(&mut buffer, &mut endec_session);

        let encoded_session = endec_session.encode_session(file_metadata);

        if let Err(e) = write_bytes(&path, &encoded_session, WriteMode::CreateOrTruncate) {
            return Err(e);
        }

        if let Err(e) = write_bytes(&path, &buffer, WriteMode::AlwaysAppend) {
            let _ = remove_file(path);  // let's not unwrap this...
            return Err(e);
        }

        Ok(())
    }

    fn load_from_file(path: &str, file_metadata: Option<u64>) -> Result<Self, EndecError> where Self: Sized {
        match read_bytes(path) {
            Ok(b) => {
                let mut index = 0;
                let mut session = EndecSession::decode_session(&b, &mut index, file_metadata).map_err(
                    |mut e| e.set_path(&path.to_string()).to_owned()
                )?;

                Self::decode(&b, &mut index, &mut session)
            },
            Err(e) => {
                return Err(e.into());
            },
        }
    }
}

impl Endec for char {
    fn encode(&self, buf: &mut Vec<u8>, session: &mut EndecSession) {
        (*self as u32).encode(buf, session);
    }

    fn decode(buf: &[u8], index: &mut usize, session: &mut EndecSession) -> Result<Self, EndecError> {
        let c = u32::decode(buf, index, session)?;

        char::from_u32(c).ok_or_else(|| EndecError::invalid_char(c))
    }
}

impl Endec for bool {
    fn encode(&self, buf: &mut Vec<u8>, _: &mut EndecSession) {
        buf.push(*self as u8);
    }

    fn decode(buf: &[u8], index: &mut usize, _: &mut EndecSession) -> Result<Self, EndecError> {
        match buf.get(*index) {
            Some(n) => {
                *index += 1;

                match *n {
                    0 => Ok(false),
                    1 => Ok(true),
                    2.. => Err(EndecError::invalid_enum_variant(*n)),
                }
            },
            None => Err(EndecError::eof()),
        }
    }
}

impl Endec for String {
    fn encode(&self, buf: &mut Vec<u8>, session: &mut EndecSession) {
        // Does this clone the inner buffer?
        (<&str as Into<Vec<u8>>>::into(&self)).encode(buf, session);
    }

    fn decode(buf: &[u8], index: &mut usize, session: &mut EndecSession) -> Result<Self, EndecError> {
        let v8 = Vec::<u8>::decode(buf, index, session)?;

        String::from_utf8(v8).map_err(|e| e.into())
    }
}

impl<T: Endec> Endec for Option<T> {
    fn encode(&self, buf: &mut Vec<u8>, session: &mut EndecSession) {
        if let Some(v) = self {
            buf.push(1);
            v.encode(buf, session);
        }

        else {
            buf.push(0);
        }
    }

    fn decode(buf: &[u8], index: &mut usize, session: &mut EndecSession) -> Result<Self, EndecError> {
        match buf.get(*index) {
            Some(n) => {
                *index += 1;

                match *n {
                    0 => Ok(None),
                    1 => Ok(Some(T::decode(buf, index, session)?)),
                    2.. => Err(EndecError::invalid_enum_variant(*n)),
                }
            },
            None => Err(EndecError::eof()),
        }
    }
}

impl<T: Endec, U: Endec> Endec for (T, U) {
    fn encode(&self, buf: &mut Vec<u8>, sess: &mut EndecSession) {
        self.0.encode(buf, sess);
        self.1.encode(buf, sess);
    }

    fn decode(buf: &[u8], index: &mut usize, sess: &mut EndecSession) -> Result<Self, EndecError> {
        Ok((
            T::decode(buf, index, sess)?,
            U::decode(buf, index, sess)?,
        ))
    }
}

impl <T: Endec> Endec for Box<T> {
    fn encode(&self, buf: &mut Vec<u8>, sess: &mut EndecSession) {
        self.as_ref().encode(buf, sess);
    }

    fn decode(buf: &[u8], index: &mut usize, sess: &mut EndecSession) -> Result<Self, EndecError> {
        Ok(Box::new(T::decode(buf, index, sess)?))
    }
}
