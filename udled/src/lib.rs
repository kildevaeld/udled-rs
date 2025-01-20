#![no_std]

extern crate alloc;

#[cfg(feature = "std")]
extern crate std;

mod cursor;
mod either;
mod error;
mod input;
mod lexeme;
mod reader;
mod span;
mod string;
pub mod token;

pub use self::{
    either::Either, error::Error, input::Input, lexeme::*, reader::Reader, span::*,
    string::StringExt, token::Tokenizer,
};
