use udled2::{or, AsChar, Buffer, Digit, Input, Reader, Test, Tokenizer, TokenizerExt};
use udled2_tokenizers::Integer;

const TWO_DIGITS: (Digit, Digit) = (Digit(10), Digit(10));
const FOUR_DIGITS: (Digit, Digit, Digit, Digit) = (Digit(10), Digit(10), Digit(10), Digit(10));

pub struct DateParser;

impl<'input, B> Tokenizer<'input, B> for DateParser
where
    B: Buffer<'input>,
    B::Item: AsChar,
{
    type Token = (u16, u8, u8);

    fn to_token(&self, reader: &mut Reader<'_, 'input, B>) -> Result<Self::Token, udled2::Error> {
        let (year, _, month, _, day) = reader.parse((
            FOUR_DIGITS.into_integer(10),
            '-',
            TWO_DIGITS.into_integer(10),
            '-',
            TWO_DIGITS.into_integer(10),
        ))?;

        Ok((year.value as _, month.value as _, day.value as _))
    }

    fn eat(&self, reader: &mut Reader<'_, 'input, B>) -> Result<(), udled2::Error> {
        reader.eat((FOUR_DIGITS, '-', TWO_DIGITS, '-', TWO_DIGITS))
    }

    fn peek(&self, reader: &mut Reader<'_, 'input, B>) -> bool {
        reader.is(Test(FOUR_DIGITS))
    }
}

pub struct TimeParser;

impl<'input, B> Tokenizer<'input, B> for TimeParser
where
    B: Buffer<'input>,
    B::Item: AsChar,
{
    type Token = (u8, u8, u8, u32);

    fn to_token(&self, reader: &mut Reader<'_, 'input, B>) -> Result<Self::Token, udled2::Error> {
        let parser = TWO_DIGITS.into_integer(10);

        let (hour, _, min, _, secs) = reader.parse((&parser, ':', &parser, ':', &parser))?;

        let nano = if reader.is(',') {
            reader.eat('.')?;
            let nano = reader.parse(Integer)?;
            nano.value as u32
        } else {
            0
        };

        Ok((hour.value as _, min.value as _, secs.value as _, nano))
    }

    fn eat(&self, reader: &mut Reader<'_, 'input, B>) -> Result<(), udled2::Error> {
        reader.eat((TWO_DIGITS, ':', TWO_DIGITS, ':', TWO_DIGITS))?;

        if reader.is(',') {
            reader.eat('.')?;
            reader.eat(Integer)?;
        }

        Ok(())
    }

    fn peek(&self, reader: &mut Reader<'_, 'input, B>) -> bool {
        reader.is(Test((TWO_DIGITS, ':')))
    }
}

pub struct TimeZoneParser;

impl<'input, B> Tokenizer<'input, B> for TimeZoneParser
where
    B: Buffer<'input>,
    B::Item: AsChar,
{
    type Token = (i32, u32, u32);

    fn to_token(&self, reader: &mut Reader<'_, 'input, B>) -> Result<Self::Token, udled2::Error> {
        or(
            // Utc
            'z'.or('Z').map_ok(|_| (0i32, 0u32, 0u32)),
            // Offset
            (
                // Sign
                '-'.map_ok(|_| -1).or('+'.map_ok(|_| 1)),
                // Hour
                TWO_DIGITS.into_integer(10).map_ok(|m| m.value as u32),
                ':'.optional(),
                // Optional Minute
                TWO_DIGITS
                    .into_integer(10)
                    .optional()
                    .map_ok(|m| m.map(|m| m.value as u32).unwrap_or(0)),
            )
                .map_ok(|(sign, h, _, m)| (sign.unify(), h, m)),
        )
        .map_ok(|m| m.unify())
        .parse(reader)
    }

    fn eat(&self, reader: &mut Reader<'_, 'input, B>) -> Result<(), udled2::Error> {
        reader.eat(or(
            // UTC
            'z'.or('Z'),
            // Offset
            (
                '-'.or('+'),
                TWO_DIGITS,
                ':'.optional(),
                TWO_DIGITS.optional(),
            ),
        ))
    }

    fn peek(&self, reader: &mut Reader<'_, 'input, B>) -> bool {
        reader.is(Test(('+'.or('-'), TWO_DIGITS)))
    }
}

pub struct DateTimeParser;

impl<'input, B> Tokenizer<'input, B> for DateTimeParser
where
    B: Buffer<'input>,
    B::Item: AsChar,
{
    type Token = (u16, u8, u8, u8, u8, u8, u32, (i32, u32, u32));

    fn to_token(&self, reader: &mut Reader<'_, 'input, B>) -> Result<Self::Token, udled2::Error> {
        let (year, month, day) = reader.parse(DateParser)?;

        reader.eat('T'.or(' '))?;

        let (h, m, s, n) = reader.parse(TimeParser)?;

        let timezone = reader.parse(TimeZoneParser)?;

        Ok((year, month, day, h, m, s, n, timezone))
    }

    fn eat(&self, reader: &mut Reader<'_, 'input, B>) -> Result<(), udled2::Error> {
        reader.eat((DateParser, 'T'.or(' '), TimeParser, TimeZoneParser))?;

        Ok(())
    }

    fn peek(&self, reader: &mut Reader<'_, 'input, B>) -> bool {
        reader.is(Test((FOUR_DIGITS, '-')))
    }
}

fn main() -> udled2::Result<()> {
    let input = "2025-09-24T20:27:35-01:30";

    let mut parser = Input::new(input.as_bytes());

    let (year, month, day, h, m, s, n, timezone) = parser.parse(DateTimeParser)?;

    println!("{}-{}-{} {}.{}.{}.{}", day, month, year, h, m, s, n);
    println!("timezone: {:?}", timezone);

    Ok(())
}
