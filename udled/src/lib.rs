#![no_std]

extern crate alloc;

pub mod buffer;
#[cfg(feature = "binary")]
pub mod bytes;
mod cursor;
mod either;
mod error;
mod ext;
mod input;
mod into_tokenizer;
mod item;
mod location;
mod macros;
mod reader;
mod span;
mod tokenizer;
mod traits;

pub mod tokenizers;

pub use self::into_tokenizer::IntoTokenizer;

pub use self::{
    buffer::{Buffer, BufferItem},
    either::Either,
    error::*,
    ext::TokenizerExt,
    input::Input,
    item::Item,
    location::Location,
    reader::Reader,
    span::*,
    tokenizer::{Char, Tokenizer, EOF},
    tokenizers::Next,
    traits::*,
};

#[cfg(feature = "macros")]
pub use udled_macros::visitor;

pub mod prelude {
    #[cfg(feature = "binary")]
    pub use super::bytes::FromBytesExt;
    pub use super::ext::TokenizerExt;
}
