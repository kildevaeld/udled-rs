use core::fmt;

use alloc::{boxed::Box, string::String};

#[derive(Debug)]
pub struct Error {
    position: usize,
    message: Box<dyn core::error::Error + Send + Sync>,
}

impl Error {
    pub fn new<T: Into<Box<dyn core::error::Error + Send + Sync>>>(
        position: usize,
        msg: T,
    ) -> Error {
        Error {
            position,
            message: msg.into(),
        }
    }
}

impl fmt::Display for Error {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        todo!()
    }
}

impl core::error::Error for Error {}

pub type Result<T> = core::result::Result<T, Error>;
