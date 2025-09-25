use core::marker::PhantomData;

use crate::{
    buffer::{Buffer, BufferItem},
    error::Error,
};

pub struct Cursor<'a, 'input, B> {
    index: &'a mut usize,
    buffer: &'a B,
    life: PhantomData<&'input ()>,
}

impl<'a, 'input, B> Cursor<'a, 'input, B> {
    pub fn new(index: &'a mut usize, buffer: &'a B) -> Cursor<'a, 'input, B> {
        Cursor {
            index,
            buffer,
            life: PhantomData,
        }
    }
}

impl<'a, 'input, B> Cursor<'a, 'input, B>
where
    B: Buffer<'input>,
{
    pub fn peek(&self) -> Option<BufferItem<'input, B>> {
        self.buffer.get(*self.index)
    }

    pub fn peekn(&self, n: usize) -> Option<BufferItem<'input, B>> {
        self.buffer.get(*self.index + n)
    }

    pub fn eat(&mut self) -> Option<BufferItem<'input, B>> {
        let ch = self.buffer.get(*self.index);
        *self.index += 1;

        // if let Some(m) = ch.as_ref().map(|(_, m)| *m) {
        //     if m == "\n" {
        //         *self.line_no += 1;
        //         *self.col_no = 1;
        //     } else {
        //         *self.col_no += 1;
        //     }
        // }

        ch
    }

    pub fn position(&self) -> usize {
        let len = self.buffer.len();
        if len == 0 {
            return 0;
        }
        self.buffer.get(*self.index).map(|m| m.index).unwrap_or(
            self.buffer
                .get(len - 1)
                .map(|m| m.index + m.len)
                .unwrap_or_default(),
        )
    }

    pub fn child<F, R>(&mut self, func: F) -> Result<R, Error>
    where
        F: FnOnce(Cursor<'_, 'input, B>) -> Result<R, Error>,
    {
        let mut next_idx = *self.index;

        let child = Cursor {
            index: &mut next_idx,
            buffer: self.buffer,
            life: PhantomData,
        };

        match func(child) {
            Ok(ret) => {
                *self.index = next_idx;

                Ok(ret)
            }
            Err(err) => Err(err),
        }
    }

    pub fn child_peek<F, R>(&mut self, func: F) -> R
    where
        F: FnOnce(Cursor<'_, 'input, B>) -> R,
    {
        let mut next_idx = *self.index;

        let child = Cursor {
            index: &mut next_idx,
            buffer: self.buffer,
            life: PhantomData,
        };

        func(child)
    }

    pub fn buffer(&self) -> &B {
        self.buffer
    }
}
