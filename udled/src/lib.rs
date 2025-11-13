#![no_std]

extern crate alloc;

mod buffer;
pub mod bytes;
mod cursor;
mod either;
mod error;
mod ext;
mod input;
mod item;
mod location;
mod macros;
mod parser;
mod reader;
mod span;
mod tokenizer;
mod traits;

mod tokenizers;

pub use self::parser::Parser;

pub use self::{
    buffer::{Buffer, BufferItem, StringBuffer},
    either::Either,
    error::*,
    ext::TokenizerExt,
    input::Input,
    item::Item,
    location::Location,
    reader::Reader,
    span::*,
    tokenizer::{Char, IgnoreCase, Not, Peek, Prefix, Tokenizer, EOF},
    tokenizers::*,
    traits::*,
};

#[cfg(feature = "macros")]
pub use udled_macros::visitor;
