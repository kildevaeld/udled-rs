use alloc::vec::Vec;

pub struct BufferItem<'a, B>
where
    B: Buffer<'a>,
{
    pub index: usize,
    pub len: usize,
    pub item: B::Item,
}

pub trait Buffer<'a>: Sized {
    type Source;
    type Item;

    fn source(&self) -> Self::Source;

    fn len(&self) -> usize;
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
