use alloc::vec::Vec;

use crate::{AsChar, Buffer, Char, Error, Item, Reader, Tokenizer};

#[derive(Debug, Clone, Copy)]
pub struct Digit(pub u32);

impl Default for Digit {
    fn default() -> Self {
        Digit(10)
    }
}

impl From<Digit> for u32 {
    fn from(value: Digit) -> Self {
        value.0
    }
}

impl<'input, S> Tokenizer<'input, S> for Digit
where
    S: Buffer<'input>,
    S::Item: AsChar,
{
    type Token = Item<u32>;

    fn to_token(&self, reader: &mut Reader<'_, 'input, S>) -> Result<Self::Token, Error> {
        let item = reader.parse(Char)?;

        item.value
            .to_digit(self.0)
            .map(|value| Item {
                span: item.span,
                value,
            })
            .ok_or_else(|| reader.error("digit"))
    }

    fn peek(&self, reader: &mut Reader<'_, 'input, S>) -> bool {
        match reader.peek_ch().and_then(|m| m.as_char()) {
            Some(char) => char.is_digit(self.0),
            None => false,
        }
    }
}

impl From<Item<u32>> for u32 {
    fn from(value: Item<u32>) -> Self {
        value.value
    }
}

pub trait AsDigits {
    type Iter: Iterator<Item = u32>;

    fn digits(self) -> Self::Iter;
}

impl AsDigits for Vec<u32> {
    type Iter = alloc::vec::IntoIter<u32>;
    fn digits(self) -> Self::Iter {
        self.into_iter()
    }
}

impl<T: Into<u32>> AsDigits for (T,) {
    type Iter = core::array::IntoIter<u32, 1>;
    fn digits(self) -> Self::Iter {
        [self.0.into()].into_iter()
    }
}

impl<T1: Into<u32>, T2: Into<u32>> AsDigits for (T1, T2) {
    type Iter = core::array::IntoIter<u32, 2>;
    fn digits(self) -> Self::Iter {
        [self.0.into(), self.1.into()].into_iter()
    }
}

impl<T1: Into<u32>, T2: Into<u32>, T3: Into<u32>> AsDigits for (T1, T2, T3) {
    type Iter = core::array::IntoIter<u32, 3>;
    fn digits(self) -> Self::Iter {
        [self.0.into(), self.1.into(), self.2.into()].into_iter()
    }
}

impl<T1: Into<u32>, T2: Into<u32>, T3: Into<u32>, T4: Into<u32>> AsDigits for (T1, T2, T3, T4) {
    type Iter = core::array::IntoIter<u32, 4>;
    fn digits(self) -> Self::Iter {
        [self.0.into(), self.1.into(), self.2.into(), self.3.into()].into_iter()
    }
}

impl<T1: Into<u32>, T2: Into<u32>, T3: Into<u32>, T4: Into<u32>, T5: Into<u32>> AsDigits
    for (T1, T2, T3, T4, T5)
{
    type Iter = core::array::IntoIter<u32, 5>;
    fn digits(self) -> Self::Iter {
        [
            self.0.into(),
            self.1.into(),
            self.2.into(),
            self.3.into(),
            self.4.into(),
        ]
        .into_iter()
    }
}

impl<T1: Into<u32>, T2: Into<u32>, T3: Into<u32>, T4: Into<u32>, T5: Into<u32>, T6: Into<u32>>
    AsDigits for (T1, T2, T3, T4, T5, T6)
{
    type Iter = core::array::IntoIter<u32, 6>;
    fn digits(self) -> Self::Iter {
        [
            self.0.into(),
            self.1.into(),
            self.2.into(),
            self.3.into(),
            self.4.into(),
            self.5.into(),
        ]
        .into_iter()
    }
}

impl<
        T1: Into<u32>,
        T2: Into<u32>,
        T3: Into<u32>,
        T4: Into<u32>,
        T5: Into<u32>,
        T6: Into<u32>,
        T7: Into<u32>,
    > AsDigits for (T1, T2, T3, T4, T5, T6, T7)
{
    type Iter = core::array::IntoIter<u32, 7>;
    fn digits(self) -> Self::Iter {
        [
            self.0.into(),
            self.1.into(),
            self.2.into(),
            self.3.into(),
            self.4.into(),
            self.5.into(),
            self.6.into(),
        ]
        .into_iter()
    }
}

impl<
        T1: Into<u32>,
        T2: Into<u32>,
        T3: Into<u32>,
        T4: Into<u32>,
        T5: Into<u32>,
        T6: Into<u32>,
        T7: Into<u32>,
        T8: Into<u32>,
    > AsDigits for (T1, T2, T3, T4, T5, T6, T7, T8)
{
    type Iter = core::array::IntoIter<u32, 8>;
    fn digits(self) -> Self::Iter {
        [
            self.0.into(),
            self.1.into(),
            self.2.into(),
            self.3.into(),
            self.4.into(),
            self.5.into(),
            self.6.into(),
            self.7.into(),
        ]
        .into_iter()
    }
}

impl<
        T1: Into<u32>,
        T2: Into<u32>,
        T3: Into<u32>,
        T4: Into<u32>,
        T5: Into<u32>,
        T6: Into<u32>,
        T7: Into<u32>,
        T8: Into<u32>,
        T9: Into<u32>,
    > AsDigits for (T1, T2, T3, T4, T5, T6, T7, T8, T9)
{
    type Iter = core::array::IntoIter<u32, 9>;
    fn digits(self) -> Self::Iter {
        [
            self.0.into(),
            self.1.into(),
            self.2.into(),
            self.3.into(),
            self.4.into(),
            self.5.into(),
            self.6.into(),
            self.7.into(),
            self.8.into(),
        ]
        .into_iter()
    }
}

impl<
        T1: Into<u32>,
        T2: Into<u32>,
        T3: Into<u32>,
        T4: Into<u32>,
        T5: Into<u32>,
        T6: Into<u32>,
        T7: Into<u32>,
        T8: Into<u32>,
        T9: Into<u32>,
        T10: Into<u32>,
    > AsDigits for (T1, T2, T3, T4, T5, T6, T7, T8, T9, T10)
{
    type Iter = core::array::IntoIter<u32, 10>;
    fn digits(self) -> Self::Iter {
        [
            self.0.into(),
            self.1.into(),
            self.2.into(),
            self.3.into(),
            self.4.into(),
            self.5.into(),
            self.6.into(),
            self.7.into(),
            self.8.into(),
            self.9.into(),
        ]
        .into_iter()
    }
}

impl<
        T1: Into<u32>,
        T2: Into<u32>,
        T3: Into<u32>,
        T4: Into<u32>,
        T5: Into<u32>,
        T6: Into<u32>,
        T7: Into<u32>,
        T8: Into<u32>,
        T9: Into<u32>,
        T10: Into<u32>,
        T11: Into<u32>,
    > AsDigits for (T1, T2, T3, T4, T5, T6, T7, T8, T9, T10, T11)
{
    type Iter = core::array::IntoIter<u32, 11>;
    fn digits(self) -> Self::Iter {
        [
            self.0.into(),
            self.1.into(),
            self.2.into(),
            self.3.into(),
            self.4.into(),
            self.5.into(),
            self.6.into(),
            self.7.into(),
            self.8.into(),
            self.9.into(),
            self.10.into(),
        ]
        .into_iter()
    }
}

impl<
        T1: Into<u32>,
        T2: Into<u32>,
        T3: Into<u32>,
        T4: Into<u32>,
        T5: Into<u32>,
        T6: Into<u32>,
        T7: Into<u32>,
        T8: Into<u32>,
        T9: Into<u32>,
        T10: Into<u32>,
        T11: Into<u32>,
        T12: Into<u32>,
    > AsDigits for (T1, T2, T3, T4, T5, T6, T7, T8, T9, T10, T11, T12)
{
    type Iter = core::array::IntoIter<u32, 12>;
    fn digits(self) -> Self::Iter {
        [
            self.0.into(),
            self.1.into(),
            self.2.into(),
            self.3.into(),
            self.4.into(),
            self.5.into(),
            self.6.into(),
            self.7.into(),
            self.8.into(),
            self.9.into(),
            self.10.into(),
            self.11.into(),
        ]
        .into_iter()
    }
}
