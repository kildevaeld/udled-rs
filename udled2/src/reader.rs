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
    pub fn error<T: Into<Box<dyn core::error::Error + Send + Sync>>>(&self, error: T) -> Error {
        Error::new(self.cursor.prev_position(), error)
    }

    pub fn error_with<T: Into<Box<dyn core::error::Error + Send + Sync>>>(
        &self,
        error: T,
        errors: Vec<Error>,
    ) -> Error {
        Error::new_with(self.cursor.prev_position(), error, errors)
    }

    pub fn position(&self) -> usize {
        self.cursor.position()
    }

    pub fn buffer(&self) -> &B {
        self.cursor.buffer()
    }

    pub fn eat_ch(&mut self) -> Result<B::Item> {
        let Some(ch) = self.cursor.eat() else {
            return Err(Error::new(self.position(), "EOF"));
        };
        Ok(ch.item)
    }

    /// Peek char at current position
    pub fn peek_ch(&mut self) -> Option<B::Item> {
        self.cursor.peek().map(|m| m.item)
    }

    /// Peek char at n position relative to current position
    pub fn peek_chn(&mut self, peek: usize) -> Option<B::Item> {
        self.cursor.peekn(peek).map(|m| m.item)
    }

    pub fn peek<T: Tokenizer<'input, B>>(&mut self, tokenizer: T) -> bool {
        self.cursor.child_peek(|cursor| {
            let mut reader = Reader { cursor };
            tokenizer.peek(&mut reader)
        })
    }

    /// Parse a token
    pub fn parse<T: Tokenizer<'input, B>>(&mut self, tokenizer: T) -> Result<T::Token> {
        self.cursor.child(|cursor| {
            let mut reader = Reader { cursor };

            let token = tokenizer.to_token(&mut reader)?;

            Ok(token)
        })
    }

    /// Eat a token
    pub fn eat<T: Tokenizer<'input, B>>(&mut self, tokenizer: T) -> Result<()> {
        self.cursor.child(|cursor| {
            let mut reader = Reader { cursor };
            tokenizer.eat(&mut reader)
        })
    }
}
