use super::File;
use std::fmt;

impl fmt::Debug for File {
    fn fmt(&self, fmt: &mut fmt::Formatter) -> Result<(), fmt::Error> {
        write!(fmt, "\"TODO: Debug for File\"")
    }
}
