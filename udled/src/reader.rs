use alloc::{borrow::Cow, vec::Vec};

use crate::{
    cursor::Cursor,
    error::{Error, Result},
    token::Tokenizer,
    Input,
};

pub struct Reader<'a, 'b> {
    cursor: Cursor<'a, 'b>,
}

impl<'a, 'b> Reader<'a, 'b> {
    pub fn new(input: &'a mut Input<'b>) -> Reader<'a, 'b> {
        Reader {
            cursor: Cursor::new(
                &input.buffer,
                &mut input.next_idx,
                &mut input.line_no,
                &mut input.col_no,
            ),
        }
    }

    /// Consume next str
    pub fn eat_ch(&mut self) -> Result<&'b str> {
        let Some((_, ch)) = self.cursor.eat() else {
            return Err(Error::new(
                "eof",
                self.position(),
                self.line_no(),
                self.col_no(),
            ));
        };

        Ok(ch)
    }

    /// Current line number
    pub fn line_no(&self) -> usize {
        self.cursor.line_no()
    }

    /// Current column
    pub fn col_no(&self) -> usize {
        self.cursor.col_no()
    }

    /// The current position
    pub fn position(&self) -> usize {
        self.cursor.position()
    }

    /// The input string
    pub fn source(&self) -> &'b str {
        self.cursor.source()
    }

    pub fn error(&self, message: impl Into<Cow<'static, str>>) -> Error {
        Error::new(message, self.position(), self.line_no(), self.col_no())
    }

    pub fn error_with(&self, message: impl Into<Cow<'static, str>>, errors: Vec<Error>) -> Error {
        Error::new_with(
            message,
            self.position(),
            self.line_no(),
            self.col_no(),
            errors,
        )
    }

    /// Peek char at current position
    pub fn peek_ch(&mut self) -> Option<&'b str> {
        self.cursor.peek().map(|m| m.1)
    }

    /// Peek char at n position relative to current position
    pub fn peek_chn(&mut self, peek: usize) -> Option<&'b str> {
        self.cursor.peekn(peek).map(|m| m.1)
    }

    /// Peek a tokenizer
    pub fn peek<T: Tokenizer>(&mut self, tokenizer: T) -> Result<bool> {
        self.cursor.child_peek(|cursor| {
            let mut reader = Reader { cursor };
            tokenizer.peek(&mut reader)
        })
    }

    /// Returns true if end of feed is reached
    pub fn eof(&self) -> bool {
        self.cursor.eof()
    }

    /// Parse a token
    pub fn parse<T: Tokenizer>(&mut self, tokenizer: T) -> Result<T::Token<'b>> {
        self.cursor.child(|cursor| {
            let mut reader = Reader { cursor };

            let token = tokenizer.to_token(&mut reader)?;

            Ok(token)
        })
    }

    /// Eat a token
    pub fn eat<T: Tokenizer>(&mut self, tokenizer: T) -> Result<()> {
        let _ = self.parse(tokenizer)?;
        Ok(())
    }
}
