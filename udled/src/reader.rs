use alloc::{boxed::Box, vec::Vec};

use crate::{
    buffer::Buffer,
    cursor::Cursor,
    error::{Error, Result},
    tokenizer::Tokenizer,
};

pub struct Reader<'a, 'input, B> {
    cursor: Cursor<'a, 'input, B>,
}

impl<'a, 'input, B> Reader<'a, 'input, B> {
    pub(crate) fn new(cursor: Cursor<'a, 'input, B>) -> Reader<'a, 'input, B> {
        Reader { cursor }
    }
}

impl<'a, 'input, B> Reader<'a, 'input, B>
where
    B: Buffer<'input>,
{
    #[inline]
    pub fn error<T: Into<Box<dyn core::error::Error + Send + Sync>>>(&self, error: T) -> Error {
        Error::new(self.cursor.prev_position(), error)
    }

    #[inline]
    pub fn error_with<T: Into<Box<dyn core::error::Error + Send + Sync>>>(
        &self,
        error: T,
        errors: Vec<Error>,
    ) -> Error {
        Error::new_with(self.cursor.prev_position(), error, errors)
    }

    #[inline]
    pub fn position(&self) -> usize {
        self.cursor.position()
    }

    #[inline]
    pub fn buffer(&self) -> &B {
        self.cursor.buffer()
    }

    #[inline]
    pub fn read(&mut self) -> Result<B::Item> {
        let Some(ch) = self.cursor.eat() else {
            return Err(Error::new(self.position(), "EOF"));
        };
        Ok(ch.item)
    }

    /// Peek char at current position
    #[inline]
    pub fn peek_ch(&mut self) -> Option<B::Item> {
        self.cursor.peek().map(|m| m.item)
    }

    /// Peek char at n position relative to current position
    #[inline]
    pub fn peek_chn(&mut self, peek: usize) -> Option<B::Item> {
        self.cursor.peekn(peek).map(|m| m.item)
    }

    #[inline]
    pub fn is<T: Tokenizer<'input, B>>(&mut self, tokenizer: T) -> bool {
        self.cursor.child_peek(|cursor| {
            let mut reader = Reader { cursor };
            tokenizer.peek(&mut reader)
        })
    }

    /// Parse a token
    #[inline]
    pub fn parse<T: Tokenizer<'input, B>>(&mut self, tokenizer: T) -> Result<T::Token> {
        self.cursor.child(|cursor| {
            let mut reader = Reader { cursor };

            let token = tokenizer.to_token(&mut reader)?;

            Ok(token)
        })
    }

    /// Eat a token
    #[inline]
    pub fn eat<T: Tokenizer<'input, B>>(&mut self, tokenizer: T) -> Result<()> {
        self.cursor.child(|cursor| {
            let mut reader = Reader { cursor };
            tokenizer.eat(&mut reader)
        })
    }
}
