use alloc::vec::Vec;
use unicode_segmentation::UnicodeSegmentation;

use crate::Error;

#[derive(Debug, Clone)]
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

    pub fn source(&self) -> &'input str {
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
    col_no: &'a mut usize,
    line_no: &'a mut usize,
}

impl<'a, 'input> Cursor<'a, 'input> {
    pub(crate) const fn new(
        buffer: &'a Buffer<'input>,
        next_idx: &'a mut usize,
        line_no: &'a mut usize,
        col_no: &'a mut usize,
    ) -> Cursor<'a, 'input> {
        Cursor {
            buffer,
            next_idx,
            line_no,
            col_no,
        }
    }

    pub fn source(&self) -> &'input str {
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

        if let Some(m) = ch.as_ref().map(|(_, m)| *m) {
            if m == "\n" {
                *self.line_no += 1;
                *self.col_no = 1;
            } else {
                *self.col_no += 1;
            }
        }

        ch
    }

    pub fn position(&self) -> usize {
        self.buffer
            .graph
            .get(*self.next_idx)
            .map(|m| m.0)
            .unwrap_or(
                self.buffer
                    .graph
                    .last()
                    .map(|m| m.0 + 1)
                    .unwrap_or_default(),
            )
    }

    pub fn line_no(&self) -> usize {
        *self.line_no
    }

    pub fn col_no(&self) -> usize {
        *self.col_no
    }

    pub fn eof(&self) -> bool {
        *self.next_idx >= self.buffer.graph.len()
    }

    pub fn child<F, R>(&mut self, func: F) -> Result<R, Error>
    where
        F: FnOnce(Cursor<'_, 'input>) -> Result<R, Error>,
    {
        let mut next_idx = *self.next_idx;
        let mut col_no = *self.col_no;
        let mut line_no = *self.line_no;

        let child = Cursor {
            next_idx: &mut next_idx,
            buffer: self.buffer,
            col_no: &mut col_no,
            line_no: &mut line_no,
        };

        match func(child) {
            Ok(ret) => {
                *self.next_idx = next_idx;
                *self.line_no = line_no;
                *self.col_no = col_no;
                Ok(ret)
            }
            Err(err) => Err(err),
        }
    }

    pub fn child_peek<F, R>(&mut self, func: F) -> Result<R, Error>
    where
        F: FnOnce(Cursor<'_, 'input>) -> Result<R, Error>,
    {
        let mut next_idx = *self.next_idx;
        let mut line_no = *self.line_no;
        let mut col_no = *self.col_no;

        let child = Cursor {
            next_idx: &mut next_idx,
            buffer: self.buffer,
            line_no: &mut line_no,
            col_no: &mut col_no,
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
        let mut line = 0;
        let mut col = 0;

        let mut cursor = Cursor::new(&buffer, &mut idx, &mut line, &mut col);

        assert_eq!(cursor.peek(), Some((0, "H")));
        assert_eq!(cursor.position(), 0);
        assert_eq!(cursor.peekn(1), Some((1, "e")));
        assert_eq!(cursor.eat(), Some((0, "H")));
        assert_eq!(cursor.position(), 1);
        assert_eq!(cursor.peek(), Some((1, "e")));
        assert_eq!(cursor.eat(), Some((1, "e")));
        assert_eq!(cursor.position(), 2)
    }
}
