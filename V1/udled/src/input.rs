use crate::{
    cursor::Buffer,
    error::{Error, Result},
    reader::Reader,
    token::Tokenizer,
};
use alloc::borrow::Cow;

#[derive(Debug, Clone)]
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
            .unwrap_or_else(|| self.buffer.len())
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
        self.reader().peek_ch()
    }

    pub fn peek<T: Tokenizer>(&mut self, tokenizer: T) -> Result<bool> {
        self.reader().peek(tokenizer)
    }

    pub fn parse<T: Tokenizer>(&mut self, tokenizer: T) -> Result<T::Token<'a>> {
        self.reader().parse(tokenizer)
    }

    pub fn eat<T: Tokenizer>(&mut self, tokenizer: T) -> Result<()> {
        self.reader().eat(tokenizer)
    }

    pub fn reader<'b>(&'b mut self) -> Reader<'b, 'a> {
        Reader::new(self)
    }

    pub fn source(&self) -> &'a str {
        self.buffer.source()
    }

    pub fn error(&self, message: impl Into<Cow<'static, str>>) -> Error {
        Error::new(message, self.position(), self.line_no, self.col_no)
    }
}

#[cfg(test)]
mod test {
    use super::Input;

    #[test]
    fn input() {
        let mut input = Input::new("  ");

        assert!(input.parse(" ").is_ok());
        assert_eq!(input.position(), 1);
        assert_eq!(input.line_no(), 1);
        assert_eq!(input.col_no(), 2);

        assert!(input.parse(" ").is_ok());
        assert_eq!(input.position(), 2);
        assert_eq!(input.line_no(), 1);
        assert_eq!(input.col_no(), 3);
    }

    #[test]
    fn input_error() {
        let mut input = Input::new("  ");

        assert!(input.parse(" ").is_ok());
        assert_eq!(input.position(), 1);
        assert_eq!(input.line_no(), 1);
        assert_eq!(input.col_no(), 2);

        assert!(input.parse("w").is_err());
        assert_eq!(input.position(), 1);
        assert_eq!(input.line_no(), 1);
        assert_eq!(input.col_no(), 2);
    }
}
