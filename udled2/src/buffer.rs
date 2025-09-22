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

    fn len(&self) -> usize;
    fn get(&self, idx: usize) -> Option<BufferItem<'a, Self>>;
}

impl<'a> Buffer<'a> for &'a str {
    type Item = char;
    type Source = &'a str;

    fn len(&self) -> usize {
        self.chars().count()
    }

    fn get(&self, idx: usize) -> Option<BufferItem<'a, Self>> {
        self.char_indices()
            .nth(idx)
            .map(|(index, item)| BufferItem {
                index,
                len: item.len_utf8(),
                item,
            })
    }
}
