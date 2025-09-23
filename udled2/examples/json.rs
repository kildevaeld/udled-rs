use udled2::{
    AsBytes, AsChar, AsStr, Buffer, Digit, Either, Input, Item, Many, Opt, Reader, Span, Tokenizer,
};

fn comma<'input, S>(reader: &mut Reader<'_, 'input, S>) -> udled2::Result<Item<char>>
where
    S: Buffer<'input, Item = char>,
{
    reader.parse(',')
}

const BRACKET_OPEN: char = '{';
const BRACKET_CLOSE: char = '}';
const COMMA: char = ',';
const BRACE_OPEN: char = '[';
const BRACE_CLOSE: char = ']';
const WS: Opt<Many<char>> = Opt(Many(' '));

pub struct Int;

impl<'input, B> Tokenizer<'input, B> for Int
where
    B: Buffer<'input> + 'input,
    B::Item: AsChar,
    B::Source: AsStr<'input>,
{
    type Token = Item<i128>;

    fn to_token(&self, reader: &mut Reader<'_, 'input, B>) -> Result<Self::Token, udled2::Error> {
        let mut val: i128 = 0;

        let start = reader.position();

        let sign = if reader.parse('-').is_ok() { -1 } else { 1 };

        let mut base = 10;
        if reader.parse("0x").is_ok() {
            base = 16
        };
        if reader.parse("0b").is_ok() {
            base = 2
        };

        loop {
            let ch = reader.parse(Digit(base))?;

            val = (base as i128) * val + (ch.value as i128);

            let Some(ch) = reader.peek_ch().and_then(|m| m.as_char()) else {
                break;
            };

            // Allow underscores as separators
            if ch == '_' {
                reader.eat_ch()?;
                continue;
            }

            if ch == '\0' {
                break;
            }

            if !ch.is_digit(base) {
                break;
            }
        }

        return Ok(Item::new(Span::new(start, reader.position()), sign * val));
    }
}

fn main() -> udled2::Result<()> {
    let mut input = Input::new("[-200]");

    input.reader().eat((BRACE_OPEN, WS))?;
    let int = input.reader().parse(Int)?;
    input.reader().eat((WS, BRACE_CLOSE))?;

    println!("{:?}", int);

    Ok(())
}
