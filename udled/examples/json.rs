use std::collections::HashMap;

use udled::{
    token::{Bool, Int, Opt, Str, Ws},
    Input, Tokenizer,
};

enum Value {
    String(String),
    Bool(bool),
    Int(i64),
    List(Vec<Value>),
    Object(HashMap<String, Value>),
    Null,
}

const WS: Opt<Ws> = Opt(Ws);

fn parse(input: &str) -> Result<Value, udled::Error> {
    let mut input = udled::Input::new(input);

    // input.parse(WS)?;

    parse_value(&mut input)
}

fn parse_value(input: &mut Input<'_>) -> Result<Value, udled::Error> {
    let Some(ch) = input.peek_ch() else {
        return Err(input.error("unexpected eof"));
    };

    match ch {
        "{" => parse_object(input),
        "[" => parse_list(input),
        "t" | "f" => {
            let bool = input.parse(Bool)?;
            Ok(Value::Bool(bool.value))
        }
        "n" => {
            let _ = input.parse("null")?;
            Ok(Value::Null)
        }
        "\"" => input.parse(JsonStringValue),
        _ => input.parse(JsonNumber),
    }
}

fn parse_object(input: &mut Input<'_>) -> Result<Value, udled::Error> {
    let _ = input.parse((WS, '{'))?;

    let mut map = HashMap::default();

    let _ = input.parse(WS)?;

    loop {
        if input.eos() {
            return Err(input.error("unexpected eof"));
        }

        let prop = input.parse(JsonString)?;
        let _ = input.parse((WS, ':', WS))?;

        let value = parse_value(input)?;

        map.insert(prop, value);

        let _ = input.parse(WS)?;

        if input.peek('}')? {
            input.parse('}')?;
            break;
        }

        let _ = input.parse((WS, ','))?;

        let _ = input.parse(WS)?;
    }

    Ok(Value::Object(map))
}

fn parse_list(input: &mut Input<'_>) -> Result<Value, udled::Error> {
    let _ = input.parse((WS, '['))?;

    let mut map = Vec::default();

    loop {
        if input.eos() {
            return Err(input.error("unexpected eof"));
        }

        let _ = input.parse(WS)?;

        if input.peek(']')? {
            input.parse(']')?;
            break;
        }

        let value = parse_value(input)?;

        map.push(value);

        if input.peek(']')? {
            input.parse(']')?;
            break;
        }

        let _ = input.parse((WS, ','))?;
    }

    Ok(Value::List(map))
}

struct JsonNumber;

// TODO: Impl floats
impl Tokenizer for JsonNumber {
    type Token<'a> = Value;
    fn to_token<'a>(
        &self,
        reader: &mut udled::Reader<'_, 'a>,
    ) -> Result<Self::Token<'a>, udled::Error> {
        let int = reader.parse(Int)?;
        Ok(Value::Int((int.value as i64).into()))
    }
}

struct JsonString;

impl Tokenizer for JsonString {
    type Token<'a> = String;
    // TODO: Impl json string parsing
    fn to_token<'a>(
        &self,
        reader: &mut udled::Reader<'_, 'a>,
    ) -> Result<Self::Token<'a>, udled::Error> {
        let output = reader.parse(Str)?;

        Ok(output.as_str().into())
    }

    fn peek<'a>(&self, reader: &mut udled::Reader<'_, '_>) -> Result<bool, udled::Error> {
        reader.peek('"')
    }
}

struct JsonStringValue;

impl Tokenizer for JsonStringValue {
    type Token<'a> = Value;
    // TODO: Impl json string parsing
    fn to_token<'a>(
        &self,
        reader: &mut udled::Reader<'_, 'a>,
    ) -> Result<Self::Token<'a>, udled::Error> {
        let output = reader.parse(Str)?;

        Ok(Value::String(output.value.to_string()))
    }

    fn peek<'a>(&self, reader: &mut udled::Reader<'_, '_>) -> Result<bool, udled::Error> {
        reader.peek('"')
    }
}

fn main() {
    let json = parse(r#"{"field": 200, "list": [true, "string"]}"#);
}
