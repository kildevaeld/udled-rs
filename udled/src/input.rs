use alloc::{borrow::Cow, vec::Vec};

use crate::{
    cursor::{Buffer, Cursor},
    error::Error,
    token::Tokenizer,
};

pub struct Input<'a> {
    buffer: Buffer<'a>,
    next_idx: usize,
    line_no: usize,
    col_no: usize,
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

    pub fn peek_ch(&mut self) -> Option<&str> {
        self.buffer.get(self.next_idx).map(|m| m.1)
    }

    pub fn peek<T: Tokenizer>(&mut self, tokenizer: T) -> Result<bool, Error> {
        let mut next_idx = self.next_idx;
        tokenizer.peek(&mut Reader {
            cursor: Cursor::new(&self.buffer, &mut next_idx),
            line_no: self.line_no,
            col_no: self.col_no,
        })
    }

    pub fn parse<T: Tokenizer>(&mut self, tokenizer: T) -> Result<T::Token<'a>, Error> {
        let mut next_idx = self.next_idx;

        let mut reader = Reader {
            cursor: Cursor::new(&self.buffer, &mut next_idx),
            line_no: self.line_no,
            col_no: self.col_no,
        };

        let token = tokenizer.to_token(&mut reader)?;

        self.line_no = reader.line_no;
        self.col_no = reader.col_no;

        self.next_idx = next_idx;

        Ok(token)
    }

    pub fn slice(&self) -> &'a str {
        self.buffer.slice()
    }

    pub fn error(&self, message: impl Into<Cow<'static, str>>) -> Error {
        Error::new(message, self.line_no, self.col_no)
    }
}

pub struct Reader<'a, 'b> {
    cursor: Cursor<'a, 'b>,
    line_no: usize,
    col_no: usize,
}

impl<'a, 'b> Reader<'a, 'b> {
    pub fn eat_ch(&mut self) -> Result<&'b str, Error> {
        let Some((_, ch)) = self.cursor.eat() else {
            return Err(Error::new("eof", self.line_no, self.col_no));
        };

        if ch == "\n" {
            self.line_no += 1;
            self.col_no = 1;
        } else {
            self.col_no += 1;
        }

        Ok(ch)
    }

    pub fn line_no(&self) -> usize {
        self.line_no
    }

    pub fn col_no(&self) -> usize {
        self.col_no
    }

    pub fn error(&self, message: impl Into<Cow<'static, str>>) -> Error {
        Error::new(message, self.line_no, self.col_no)
    }

    pub fn error_with(&self, message: impl Into<Cow<'static, str>>, errors: Vec<Error>) -> Error {
        Error::new_with(message, self.line_no, self.col_no, errors)
    }

    pub fn input(&self) -> &'b str {
        self.cursor.input()
    }

    pub fn position(&self) -> usize {
        self.cursor.position()
    }

    pub fn next_position(&self) -> usize {
        self.cursor.next_position()
    }

    pub fn peek_ch(&mut self) -> Option<&'b str> {
        self.cursor.peek().map(|m| m.1)
    }

    pub fn peek_chn(&mut self, peek: usize) -> Option<&'b str> {
        self.cursor.peekn(peek).map(|m| m.1)
    }

    pub fn peek<T: Tokenizer>(&mut self, tokenizer: T) -> Result<bool, Error> {
        self.cursor.child_peek(|cursor| {
            let mut reader = Reader {
                cursor,
                line_no: self.line_no,
                col_no: self.col_no,
            };

            tokenizer.peek(&mut reader)
        })
    }

    pub fn eof(&self) -> bool {
        self.cursor.eof()
    }

    pub fn parse<T: Tokenizer>(&mut self, tokenizer: T) -> Result<T::Token<'b>, Error> {
        self.cursor.child(|cursor| {
            let mut reader = Reader {
                cursor,
                line_no: self.line_no,
                col_no: self.col_no,
            };

            let token = tokenizer.to_token(&mut reader)?;

            self.line_no = reader.line_no;
            self.col_no = reader.col_no;

            Ok(token)
        })
    }

    pub fn eat<T: Tokenizer>(&mut self, tokenizer: T) -> Result<(), Error> {
        let _ = self.parse(tokenizer)?;
        Ok(())
    }
}
