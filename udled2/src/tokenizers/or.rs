use core::marker::PhantomData;

use alloc::{fmt, vec};

use crate::{Buffer, Either, Error, Reader, Tokenizer};

/// Match either L or R
pub struct Or<L, R, B> {
    left: L,
    right: R,
    buffer: PhantomData<fn(B)>,
}

pub fn or<L, R, B>(left: L, right: R) -> Or<L, R, B> {
    Or::new(left, right)
}

impl<L: fmt::Debug, R: fmt::Debug, B> fmt::Debug for Or<L, R, B> {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        f.debug_struct("Or")
            .field("left", &self.left)
            .field("right", &self.right)
            .finish()
    }
}

impl<L: Clone, R: Clone, B> Clone for Or<L, R, B> {
    fn clone(&self) -> Self {
        Or {
            left: self.left.clone(),
            right: self.right.clone(),
            buffer: PhantomData,
        }
    }
}

impl<L: Copy, R: Copy, B> Copy for Or<L, R, B> {}

impl<L, R, B> Or<L, R, B> {
    pub const fn new(left: L, right: R) -> Or<L, R, B> {
        Or {
            left,
            right,
            buffer: PhantomData,
        }
    }
}

impl<'input, L, R, B> Tokenizer<'input, B> for Or<L, R, B>
where
    L: Tokenizer<'input, B>,
    R: Tokenizer<'input, B>,
    B: Buffer<'input>,
{
    type Token = Either<L::Token, R::Token>;
    fn to_token<'a>(&self, reader: &mut Reader<'_, 'input, B>) -> Result<Self::Token, Error> {
        let left_err = match reader.parse(&self.left) {
            Ok(ret) => return Ok(Either::Left(ret)),
            Err(err) => err,
        };

        let right_err = match reader.parse(&self.right) {
            Ok(ret) => return Ok(Either::Right(ret)),
            Err(err) => err,
        };

        Err(reader.error_with("either", vec![left_err, right_err]))
    }

    fn eat(&self, reader: &mut Reader<'_, 'input, B>) -> Result<(), Error> {
        let left_err = match reader.eat(&self.left) {
            Ok(_) => return Ok(()),
            Err(err) => err,
        };

        let right_err = match reader.eat(&self.right) {
            Ok(_) => return Ok(()),
            Err(err) => err,
        };

        Err(reader.error_with("either", vec![left_err, right_err]))
    }

    fn peek(&self, reader: &mut Reader<'_, 'input, B>) -> bool {
        reader.is(&self.left) || reader.is(&self.right)
    }
}
