use alloc::vec::Vec;
use unicode_segmentation::UnicodeSegmentation;

use crate::Error;

pub struct Buffer<'input> {
    input: &'input str,
    graph: Vec<(usize, &'input str)>,
}

impl<'input> Buffer<'input> {
    pub fn new(input: &'input str) -> Buffer<'input> {
        Buffer {
            input,
            graph: input.grapheme_indices(true).collect(),
        }
    }

    pub fn slice(&self) -> &'input str {
        self.input
    }

    pub fn len(&self) -> usize {
        self.graph.len()
    }

    pub fn get(&self, idx: usize) -> Option<(usize, &'input str)> {
        self.graph.get(idx).copied()
    }
}

pub struct Cursor<'a, 'input> {
    buffer: &'a Buffer<'input>,
    next_idx: &'a mut usize,
}

impl<'a, 'input> Cursor<'a, 'input> {
    pub(crate) fn new(buffer: &'a Buffer<'input>, next_idx: &'a mut usize) -> Cursor<'a, 'input> {
        Cursor { buffer, next_idx }
    }

    pub fn input(&self) -> &'input str {
        self.buffer.input
    }

    pub fn peek(&self) -> Option<(usize, &'input str)> {
        self.peekn(0)
    }

    pub fn peekn(&self, n: usize) -> Option<(usize, &'input str)> {
        self.buffer.graph.get(*self.next_idx + n).copied()
    }

    pub fn eat(&mut self) -> Option<(usize, &'input str)> {
        let ch = self.buffer.graph.get(*self.next_idx).copied();
        *self.next_idx += 1;
        ch
    }

    pub fn current_idx(&self) -> usize {
        if *self.next_idx == 0 {
            0
        } else {
            *self.next_idx - 1
        }
    }

    pub fn current(&self) -> Option<(usize, &'input str)> {
        self.buffer.graph.get(self.current_idx()).copied()
    }

    pub fn position(&self) -> usize {
        self.current().map(|m| m.0).unwrap_or_default()
    }

    pub fn next_position(&self) -> usize {
        self.buffer
            .get(*self.next_idx)
            .map(|m| m.0)
            .unwrap_or_else(|| self.position() + 1)
    }

    pub fn eof(&self) -> bool {
        *self.next_idx >= self.buffer.graph.len()
    }

    pub fn child<F, R>(&mut self, mut func: F) -> Result<R, Error>
    where
        F: FnMut(Cursor<'_, 'input>) -> Result<R, Error>,
    {
        let mut next_idx = *self.next_idx;

        let child = Cursor {
            next_idx: &mut next_idx,
            buffer: self.buffer,
        };

        match func(child) {
            Ok(ret) => {
                *self.next_idx = next_idx;
                Ok(ret)
            }
            Err(err) => Err(err),
        }
    }

    pub fn child_peek<F, R>(&mut self, mut func: F) -> Result<R, Error>
    where
        F: FnMut(Cursor<'_, 'input>) -> Result<R, Error>,
    {
        let mut next_idx = *self.next_idx;

        let child = Cursor {
            next_idx: &mut next_idx,
            buffer: self.buffer,
        };

        match func(child) {
            Ok(ret) => Ok(ret),
            Err(err) => Err(err),
        }
    }
}

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn buffer() {
        let buffer = Buffer::new("Hello");

        assert_eq!(buffer.get(0), Some((0, "H")));
        assert_eq!(buffer.get(4), Some((4, "o")));
        assert_eq!(buffer.get(5), None);
    }

    #[test]
    fn cursor() {
        let buffer = Buffer::new("Hello");

        let mut idx = 0;

        let mut cursor = Cursor::new(&buffer, &mut idx);

        assert_eq!(cursor.peek(), Some((0, "H")));
        assert_eq!(cursor.position(), 0);
        assert_eq!(cursor.peekn(1), Some((1, "e")));
        assert_eq!(cursor.eat(), Some((0, "H")));
        assert_eq!(cursor.position(), 0);
        assert_eq!(cursor.peek(), Some((1, "e")));
        assert_eq!(cursor.eat(), Some((1, "e")));
        assert_eq!(cursor.position(), 1)
    }
}
