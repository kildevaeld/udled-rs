use alloc::{borrow::Cow, vec::Vec};
use core::fmt;

pub type Result<T> = core::result::Result<T, Error>;

#[derive(Debug)]
pub struct Error {
    message: Cow<'static, str>,
    line_no: usize,
    col_no: usize,
    errors: Vec<Error>,
    position: usize,
}

impl fmt::Display for Error {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        if self.errors.is_empty() {
            write!(f, "@{}:{}: {}", self.line_no, self.col_no, self.message)
        } else {
            write!(
                f,
                "@{}:{}: {}, errors: ",
                self.line_no, self.col_no, self.message
            )?;

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

#[cfg(feature = "std")]
impl std::error::Error for Error {}

impl Error {
    pub fn new(
        message: impl Into<Cow<'static, str>>,
        position: usize,
        line_no: usize,
        col_no: usize,
    ) -> Error {
        Error {
            message: message.into(),
            line_no,
            col_no,
            errors: Default::default(),
            position,
        }
    }

    pub(crate) fn new_with(
        message: impl Into<Cow<'static, str>>,
        position: usize,
        line_no: usize,
        col_no: usize,
        errors: Vec<Error>,
    ) -> Error {
        Error {
            message: message.into(),
            line_no,
            col_no,
            errors,
            position,
        }
    }

    pub fn col_no(&self) -> usize {
        self.col_no
    }

    pub fn line_no(&self) -> usize {
        self.line_no
    }

    pub fn position(&self) -> usize {
        self.position
    }
}
