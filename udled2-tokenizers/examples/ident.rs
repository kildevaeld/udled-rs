use udled2::Input;
use udled2_tokenizers::{Ident, XmlIdent};

fn main() {
    let mut input = Input::new("html-div");
    let ident = input.parse(XmlIdent);

    println!("indent {:?}", ident)
}
