use crate::{buffer::IntoBuffer, cursor::Cursor, Buffer, Reader, Result, Tokenizer};

pub struct Input<B> {
    buffer: B,
    index: usize,
}

impl Input<()> {
    pub fn new<'a, B: IntoBuffer<'a>>(buffer: B) -> Input<B::Buffer> {
        Input {
            buffer: buffer.into_buffer(),
            index: 0,
        }
    }
}

impl<B> Input<B> {
    pub fn buffer(&self) -> &B {
        &self.buffer
    }

    #[inline(always)]
    fn reader<'this, 'input>(&'this mut self) -> Reader<'this, 'input, B>
    where
        B: Buffer<'input>,
    {
        Reader::new(Cursor::new(&mut self.index, &mut self.buffer))
    }

    pub fn is<'input, T: Tokenizer<'input, B>>(&mut self, tokenizer: T) -> bool
    where
        B: Buffer<'input>,
    {
        self.reader().is(tokenizer)
    }

    /// Parse a
    pub fn parse<'input, T: Tokenizer<'input, B>>(&mut self, tokenizer: T) -> Result<T::Token>
    where
        B: Buffer<'input>,
    {
        self.reader().parse(tokenizer)
    }

    /// Eat a token
    pub fn eat<'input, T: Tokenizer<'input, B>>(&mut self, tokenizer: T) -> Result<()>
    where
        B: Buffer<'input>,
    {
        self.reader().eat(tokenizer)
    }
}
