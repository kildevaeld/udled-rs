use std::collections::BTreeMap;

use udled::{
    AsChar, AsSlice, AsStr, Buffer, Error, Input, Location, Parser, Reader, Span, Test,
    TokenizerExt,
};
use udled_tokenizers::{Bool, Float, Integer, Str};

const BRACKET_OPEN: char = '{';
const BRACKET_CLOSE: char = '}';
const COMMA: char = ',';
const BRACE_OPEN: char = '[';
const BRACE_CLOSE: char = ']';

fn whitespace<'input, B>(reader: &mut Reader<'_, 'input, B>) -> Result<Span, Error>
where
    B: Buffer<'input>,
    B::Item: AsChar,
    B::Source: AsSlice<'input>,
{
    reader.parse(' '.or('\n').many().optional().spanned())
}

fn array<'input, B>(reader: &mut Reader<'_, 'input, B>) -> Result<Value<'input>, Error>
where
    B: Buffer<'input>,
    B::Item: AsChar,
    B::Source: AsSlice<'input> + AsStr<'input>,
    <B::Source as AsSlice<'input>>::Slice: AsStr<'input>,
{
    let ws = whitespace.parser();

    let output = reader
        .parse((
            BRACE_OPEN,
            &ws,
            value
                .parser()
                .punctuated(Test((&ws, COMMA, &ws)))
                .map_ok(|m| m.into_items().collect::<Vec<_>>()),
            &ws,
            BRACE_CLOSE.map_err(|m, buffer: &B| {
                format!(
                    "Unexpected char {:?}",
                    buffer
                        .get(m)
                        .map(|m| m.item)
                        .and_then(|m| m.as_char())
                        .unwrap_or(' ')
                )
            }),
        ))
        .map(|m| m.2)?;

    Ok(Value::List(output))
}

fn object<'input, B>(reader: &mut Reader<'_, 'input, B>) -> Result<Value<'input>, Error>
where
    B: Buffer<'input>,
    B::Item: AsChar,
    B::Source: AsSlice<'input> + AsStr<'input>,
    <B::Source as AsSlice<'input>>::Slice: AsStr<'input>,
{
    let ws = whitespace.parser();
    reader.eat(BRACKET_OPEN)?;

    reader.eat(&ws)?;

    let output = (Str, (&ws, ':', &ws), value.parser())
        .punctuated(Test((&ws, COMMA, &ws)))
        .map_ok(|m| {
            m.into_items()
                .map(|m| (m.0.value.as_str(), m.2))
                .collect::<BTreeMap<_, _>>()
        })
        .parse(reader)?;

    reader.eat(&ws)?;

    reader.eat(BRACKET_CLOSE)?;

    Ok(Value::Map(output))
}

fn value<'input, B>(reader: &mut Reader<'_, 'input, B>) -> udled::Result<Value<'input>>
where
    B: Buffer<'input>,
    B::Item: AsChar,
    B::Source: AsSlice<'input> + AsStr<'input>,
    <B::Source as AsSlice<'input>>::Slice: AsStr<'input>,
{
    if reader.is(BRACE_OPEN) {
        reader.parse(array.parser())
    } else if reader.is(BRACKET_OPEN) {
        reader.parse(object.parser())
    } else if reader.is(Str) {
        let str = reader.parse(Str)?;
        Ok(Value::String(str.value.as_str().into()))
    } else if reader.is(Bool) {
        let bool = reader.parse(Bool)?;
        Ok(Value::Bool(bool.value))
    } else if reader.is(Float) {
        let float = reader.parse(Float)?;
        Ok(Value::Float(float.value))
    } else if reader.is(Integer) {
        let int = reader.parse(Integer)?;
        Ok(Value::Int(int.value as _))
    } else {
        Err(reader.error("string, number, bool, object or array"))
    }
}

#[derive(Debug)]
pub enum Value<'a> {
    Int(i64),
    Float(f64),
    String(&'a str),
    Bool(bool),
    List(Vec<Value<'a>>),
    Map(BTreeMap<&'a str, Value<'a>>),
}

const JSON: &str = r#"{
    "name": "Wilbur\ntest",
    "age": 16,
    "favorites": ["food", "sleeping"]
}"#;

fn main() -> udled::Result<()> {
    // let mut input =
    //     Input::new("[-200 ,  440,42,\n 1000, true, \"Hello, World!\", { \"test\": 203.02e21 } ]");

    let mut input = Input::new(JSON);

    let ret = input.reader().parse(value.parser());

    let array = match ret {
        Ok(ret) => ret,
        Err(err) => {
            let location = Location::from(JSON, err.position()).unwrap();

            println!("{err}");
            println!("location: {:?}", location);

            return Ok(());
        }
    };

    println!("{:#?}", array);

    Ok(())
}
