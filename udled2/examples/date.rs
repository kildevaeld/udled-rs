use udled2::{Digit, Input, Parser, Reader, TokenizerExt};

fn date<'input>(reader: &mut Reader<'_, 'input, &'input [u8]>) -> udled2::Result<(u16, u8, u8)> {
    let digit = Digit::default();

    let year = reader.parse((&digit, &digit, &digit, &digit).into_integer(10))?;

    reader.eat('-')?;

    let month = reader.parse((&digit, &digit).into_integer(10))?;

    reader.eat('-')?;

    let day = reader.parse((&digit, &digit).into_integer(10))?;

    Ok((year.value as _, month.value as _, day.value as _))
}

fn time<'input>(reader: &mut Reader<'_, 'input, &'input [u8]>) -> udled2::Result<(u8, u8, u8)> {
    let digit = Digit::default();

    let parser = (&digit, &digit).into_integer(10);

    let (hour, _, min, _, secs) = reader.parse((&parser, ':', &parser, ':', &parser))?;

    Ok((hour.value as _, min.value as _, secs.value as _))
}

fn date_time<'input>(
    reader: &mut Reader<'_, 'input, &'input [u8]>,
) -> udled2::Result<(u16, u8, u8, u8, u8, u8)> {
    let (year, month, day) = reader.parse(date.parser())?;

    reader.eat('T'.or(' '))?;

    let (h, m, s) = reader.parse(time.parser())?;

    Ok((year, month, day, h, m, s))
}

fn main() -> udled2::Result<()> {
    let input = "2025-09-24T20:27:35";

    let mut parser = Input::new(input.as_bytes());

    let (year, month, day, h, m, s) = parser.parse(date_time.parser())?;

    println!("{}-{}-{} {}.{}.{}", day, month, year, h, m, s);

    Ok(())
}
