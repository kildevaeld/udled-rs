use alloc::vec::Vec;

pub struct BufferItem<'a, B>
where
    B: Buffer<'a>,
{
    /// Index in buffer
    pub index: usize,

    pub item: B::Item,
    /// The number of bytes the item represents in the buffer
    pub len: usize,
}

pub trait Buffer<'a>: Sized {
    type Source;
    type Item;

    fn source(&self) -> Self::Source;
    /// Number of items in the buffer
    fn len(&self) -> usize;
    /// Get the item at [idx]
    fn get(&self, idx: usize) -> Option<BufferItem<'a, Self>>;
}

pub struct StringBuffer<'a> {
    input: &'a str,
    chars: Vec<(usize, char)>,
}

impl<'a> StringBuffer<'a> {
    pub fn new(input: &'a str) -> StringBuffer<'a> {
        let chars = input.char_indices().collect();
        StringBuffer { input, chars }
    }
}

impl<'a> Buffer<'a> for StringBuffer<'a> {
    type Source = &'a str;

    type Item = char;

    fn source(&self) -> Self::Source {
        self.input
    }

    fn len(&self) -> usize {
        self.chars.len()
    }

    fn get(&self, idx: usize) -> Option<BufferItem<'a, Self>> {
        self.chars.get(idx).map(|(index, item)| BufferItem {
            index: *index,
            len: item.len_utf8(),
            item: *item,
        })
    }
}

impl<'a> Buffer<'a> for &'a [u8] {
    type Source = &'a [u8];

    type Item = u8;

    fn source(&self) -> Self::Source {
        self
    }

    fn len(&self) -> usize {
        (*self).len()
    }

    fn get(&self, idx: usize) -> Option<BufferItem<'a, Self>> {
        (*self).get(idx).map(|item| BufferItem {
            index: idx,
            len: 1,
            item: *item,
        })
    }
}

pub trait IntoBuffer<'a> {
    type Buffer: Buffer<'a>;

    fn into_buffer(self) -> Self::Buffer;
}

impl<'a> IntoBuffer<'a> for &'a str {
    type Buffer = StringBuffer<'a>;
    fn into_buffer(self) -> Self::Buffer {
        StringBuffer::new(self)
    }
}

impl<'a> IntoBuffer<'a> for StringBuffer<'a> {
    type Buffer = Self;

    fn into_buffer(self) -> Self::Buffer {
        self
    }
}

impl<'a> IntoBuffer<'a> for &'a [u8] {
    type Buffer = &'a [u8];
    fn into_buffer(self) -> Self::Buffer {
        self
    }
}
