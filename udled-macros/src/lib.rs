mod codegen;
mod parse;
mod util;

use proc_macro::TokenStream;

#[proc_macro]
pub fn precedence(input: TokenStream) -> TokenStream {
    let pratt = parse::parse(input.into()).expect("parse");
    codegen::create(pratt).into()
}
