# Udled

Udled is a lexer and parser for the rust programming language.

```rust
use udled::{Tokenizer, Input, Lex, Error, token::{Alphabetic, Ws, Punctuation, OneOrMany, Spanned}, Reader, Span};

struct Word;

impl Tokenizer for Word {
    type Token<'a> = Lex<'a>;

    fn to_token<'a>(&self, reader: &mut Reader<'_, 'a>) -> Result<Self::Token<'a>, Error> {
        let span = reader.parse(Spanned(OneOrMany(Alphabetic)))
        Ok(Lex::new(span.slice(reader.input()).unwrap(), span))
    }

    fn peek<'a>(&self, reader: &mut Reader<'_, '_>) -> Result<bool, Error> {
        reader.peek(Alphabetic)
    }
}

fn main() {

  let mut input = Input::new("Hello, World!");

  let (greeting ,_ ,_ , subject, _) = input.parse((Word, Punctuation, Ws, Word, Punctuation));

  assert_eq!(greeting.as_str(), "Hello");
  assert_eq!(subject.as_str(), "World);

}

```