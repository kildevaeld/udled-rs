use udled2::{
    or, AsChar, AsSlice, AsStr, Buffer, Digit, Error, Input, Item, Many, Opt, Or, Parser,
    PunctuatedList, Puntuated, Reader, Span, Spanned, Tokenizer, TokenizerExt,
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
const WS: Spanned<Opt<Many<char>>> = Spanned(Opt(Many(' ')));

fn whitespace<'input, B>(reader: &mut Reader<'_, 'input, B>) -> Result<Span, Error>
where
    B: Buffer<'input>,
    B::Item: AsChar,
    B::Source: AsSlice<'input>,
{
    reader.parse(Spanned(Opt(' '.or('\n').many())))
}

fn array<'input, B>(
    reader: &mut Reader<'_, 'input, B>,
) -> Result<Item<PunctuatedList<Item<i128>, (Span, Item<char>, Span)>>, Error>
where
    B: Buffer<'input>,
    B::Item: AsChar,
    B::Source: AsSlice<'input> + AsStr<'input>,
{
    let ws = whitespace.parser();
    let start = reader.parse(BRACE_OPEN)?;

    let output = reader.parse(Puntuated::new(Int, (&ws, COMMA, &ws)))?;

    reader.eat(WS)?;

    let end = reader.parse(BRACE_CLOSE)?;

    Ok(Item::new(start.span + end.span, output))
}

pub struct Int;

impl<'input, B> Tokenizer<'input, B> for Int
where
    B: Buffer<'input>,
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
    let mut input = Input::new("[-200 ,  440,42,\n 1000 ]");

    let array = input.reader().parse(array.parser())?;

    println!("{:#?}", array.value);

    Ok(())
}
