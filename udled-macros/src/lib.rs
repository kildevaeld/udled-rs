mod visitor;

use proc_macro::TokenStream;

#[proc_macro_attribute]
pub fn visitor(attr: TokenStream, item: TokenStream) -> TokenStream {
    visitor::visitor(attr, item)
}
