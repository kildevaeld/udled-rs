use udled::Input;
use udled_tokenizers::XmlIdent;

fn main() {
    let mut input = Input::new("html-div");
    let ident = input.parse(XmlIdent);

    println!("indent {:?}", ident)
}
