use core::fmt;

use alloc::{boxed::Box, vec::Vec};

#[derive(Debug)]
pub struct Error {
    position: usize,
    message: Box<dyn core::error::Error + Send + Sync>,
    errors: Vec<Error>,
}

impl Error {
    pub fn new<T: Into<Box<dyn core::error::Error + Send + Sync>>>(
        position: usize,
        msg: T,
    ) -> Error {
        Error {
            position,
            message: msg.into(),
            errors: Vec::new(),
        }
    }

    pub fn new_with<T: Into<Box<dyn core::error::Error + Send + Sync>>>(
        position: usize,
        msg: T,
        errors: Vec<Error>,
    ) -> Error {
        Error {
            position,
            message: msg.into(),
            errors,
        }
    }

    pub fn position(&self) -> usize {
        self.position
    }
}

impl fmt::Display for Error {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        if self.errors.is_empty() {
            write!(f, "@{}: {}", self.position, self.message)
        } else {
            write!(f, "@{}: {}, errors: ", self.position, self.message)?;

            for (k, v) in self.errors.iter().enumerate() {
                if k > 0 {
                    write!(f, ", ")?;
                }

                write!(f, "{}", v)?;
            }

            Ok(())
        }
    }
}

impl core::error::Error for Error {
    fn cause(&self) -> Option<&dyn core::error::Error> {
        Some(&*self.message)
    }
}

pub type Result<T> = core::result::Result<T, Error>;
