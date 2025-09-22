use crate::{
    buffer::Buffer,
    cursor::Cursor,
    error::{Error, Result},
    tokenizer::Tokenizer,
};

pub struct Reader<'a, 'input, B> {
    cursor: Cursor<'a, 'input, B>,
}

impl<'a, 'input, B> Reader<'a, 'input, B>
where
    B: Buffer<'input>,
{
    pub fn eat_ch(&mut self) -> Result<B::Item> {
        let Some(ch) = self.cursor.eat() else { todo!() };
        Ok(ch.item)
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
