use byteorder::{BigEndian, ByteOrder, LittleEndian};
use core::marker::PhantomData;

use crate::{AsBytes, AsSlice, Buffer, Item, Next, Reader, Result, Span, Tokenizer, TokenizerExt};

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum Endian {
    Lt,
    Big,
}

impl Endian {
    pub const fn native() -> Endian {
        #[cfg(target_endian = "little")]
        let byteorder = Endian::Lt;

        #[cfg(target_endian = "big")]
        let byteorder = Endian::Big;

        byteorder
    }

    pub const fn network() -> Endian {
        Self::Big
    }
}

pub struct Binary<T, B> {
    parser: PhantomData<fn(B) -> T>,
    byteorder: Endian,
}

impl<T, B> Clone for Binary<T, B> {
    fn clone(&self) -> Self {
        *self
    }
}

impl<T, B> Copy for Binary<T, B> {}

impl<T, B> Binary<T, B> {
    pub const fn new(byteorder: Endian) -> Binary<T, B> {
        Binary {
            parser: PhantomData,
            byteorder,
        }
    }

    pub const fn native() -> Binary<T, B> {
        Self::new(Endian::native())
    }

    pub const fn lt() -> Binary<T, B> {
        Self::new(Endian::Lt)
    }

    pub const fn big() -> Binary<T, B> {
        Self::new(Endian::Big)
    }
}

impl<'input, T, B> Tokenizer<'input, B> for Binary<T, B>
where
    T: FromBytes<'input, B>,
    B: Buffer<'input, Item = u8>,
{
    type Token = Item<T>;

    fn to_token(
        &self,
        reader: &mut Reader<'_, 'input, B>,
    ) -> core::result::Result<Self::Token, crate::Error> {
        let start = reader.position();
        let item = T::parse(reader, self.byteorder)?;
        let end = reader.position();
        Ok(Item::new(Span::new(start, end), item))
    }

    fn eat(&self, reader: &mut Reader<'_, 'input, B>) -> core::result::Result<(), crate::Error> {
        T::eat(reader, self.byteorder)
    }

    fn peek(&self, reader: &mut Reader<'_, 'input, B>) -> bool {
        T::is(reader, self.byteorder)
    }
}

pub trait FromBytes<'input, B>: Sized
where
    B: Buffer<'input, Item = u8>,
{
    fn parse(reader: &mut Reader<'_, 'input, B>, byteorder: Endian) -> Result<Self>;

    fn eat(reader: &mut Reader<'_, 'input, B>, byteorder: Endian) -> Result<()> {
        Self::parse(reader, byteorder)?;
        Ok(())
    }

    fn is(reader: &mut Reader<'_, 'input, B>, byteorder: Endian) -> bool {
        Self::eat(reader, byteorder).is_ok()
    }
}

macro_rules! primitives {
    ($($ty: ty => $method: ident),*) => {
      $(
        impl<'input, B> FromBytes<'input, B> for $ty
        where
            B: Buffer<'input, Item = u8>,
            B::Source: AsSlice<'input>,
            <B::Source as AsSlice<'input>>::Slice: AsBytes<'input>,
        {
            fn parse(reader: &mut Reader<'_, 'input, B>, byteorder: Endian) -> Result<Self> {
                let slice = reader.parse(Next.repeat(size_of::<$ty>() as _).slice())?;

                Ok(match byteorder {
                    Endian::Big => BigEndian::$method(slice.value.as_bytes()),
                    Endian::Lt => LittleEndian::$method(slice.value.as_bytes()),
                })
            }

            fn eat(reader: &mut Reader<'_, 'input, B>, _byteorder: Endian) -> Result<()> {
              reader.eat(Next.repeat(size_of::<$ty>() as _))?;
              Ok(())
            }


        }
      )*
    };
}

primitives!(
  i16 => read_i16,
  u16 => read_u16,
  i32 => read_i32,
  u32 => read_u32,
  i64 => read_i64,
  u64 => read_u64,
  f32 => read_f32,
  f64 => read_f64
);

impl<'input, B> FromBytes<'input, B> for u8
where
    B: Buffer<'input, Item = u8>,
{
    fn parse(reader: &mut Reader<'_, 'input, B>, _byteorder: Endian) -> Result<Self> {
        reader.read()
    }
}

impl<'input, B> FromBytes<'input, B> for i8
where
    B: Buffer<'input, Item = u8>,
{
    fn parse(reader: &mut Reader<'_, 'input, B>, _byteorder: Endian) -> Result<Self> {
        let item = reader.read()?;
        Ok(item as _)
    }
}

pub trait FromBytesExt<'input, B>: FromBytes<'input, B>
where
    B: Buffer<'input, Item = u8>,
{
    fn byteorder(endian: Endian) -> Binary<Self, B> {
        Binary::new(endian)
    }

    fn lt() -> Binary<Self, B> {
        Binary::lt()
    }

    fn big() -> Binary<Self, B> {
        Binary::big()
    }

    fn native() -> Binary<Self, B> {
        Binary::native()
    }
}

impl<'input, T, B> FromBytesExt<'input, B> for T
where
    T: FromBytes<'input, B>,
    B: Buffer<'input, Item = u8>,
{
}
