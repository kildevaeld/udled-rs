use udled2::{Buffer, Digit, Input, Item, Reader};

fn comma<'input, S>(reader: &mut Reader<'_, 'input, S>) -> udled2::Result<Item<char>>
where
    S: Buffer<'input, Item = char>,
{
    reader.parse(',')
}

fn main() -> udled2::Result<()> {
    let mut input = Input::new("øtrue,200]");

    let ret = input.reader().parse('ø')?;
    let boolean = input.reader().parse("true")?;

    input.reader().parse(',')?;

    let digit = Digit::default();

    let two = input.reader().parse(&digit)?;
    let zero = input.reader().parse(&digit)?;
    let zero = input.reader().parse(&digit)?;

    println!("Span {:?} {:?} {:?}", boolean, two, zero);

    Ok(())
}
