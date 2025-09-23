use core::fmt;

#[derive(Debug)]
pub struct Error {
    position: usize,
    message: String,
}

impl fmt::Display for Error {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        todo!()
    }
}

impl core::error::Error for Error {}

pub type Result<T> = core::result::Result<T, Error>;
