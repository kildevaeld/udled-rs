use crate::{
    cursor::Buffer,
    error::{Error, Result},
    reader::Reader,
    token::Tokenizer,
};
use alloc::borrow::Cow;

pub struct Input<'a> {
    pub(super) buffer: Buffer<'a>,
    pub(super) next_idx: usize,
    pub(super) line_no: usize,
    pub(super) col_no: usize,
}

impl<'a> Input<'a> {
    pub fn new(input: &'a str) -> Input<'a> {
        Input {
            buffer: Buffer::new(input),
            next_idx: 0,
            line_no: 1,
            col_no: 1,
        }
    }

    pub fn line_no(&self) -> usize {
        self.line_no
    }

    pub fn col_no(&self) -> usize {
        self.col_no
    }

    pub fn position(&self) -> usize {
        self.buffer
            .get(self.next_idx)
            .map(|m| m.0)
            .unwrap_or_default()
    }

    pub fn eos(&self) -> bool {
        self.next_idx >= self.buffer.len()
    }

    pub fn reset(&mut self) {
        self.next_idx = 0;
        self.col_no = 1;
        self.line_no = 1;
    }

    pub fn peek_ch(&mut self) -> Option<&str> {
        self.buffer.get(self.next_idx).map(|m| m.1)
    }

    pub fn peek<T: Tokenizer>(&mut self, tokenizer: T) -> Result<bool> {
        Reader::new(self).peek(tokenizer)
    }

    pub fn parse<T: Tokenizer>(&mut self, tokenizer: T) -> Result<T::Token<'a>> {
        Reader::new(self).parse(tokenizer)
    }

    pub fn eat<T: Tokenizer>(&mut self, tokenizer: T) -> Result<()> {
        let _ = self.parse(tokenizer)?;
        Ok(())
    }

    pub fn slice(&self) -> &'a str {
        self.buffer.slice()
    }

    pub fn error(&self, message: impl Into<Cow<'static, str>>) -> Error {
        Error::new(message, self.position(), self.line_no, self.col_no)
    }
}
