use crate::{buffer::IntoBuffer, Buffer, Cursor, Reader};

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

    pub fn reader<'this, 'input>(&'this mut self) -> Reader<'this, 'input, B>
    where
        B: Buffer<'input>,
    {
        Reader::new(Cursor::new(&mut self.index, &mut self.buffer))
    }
}
