use proc_macro2::{Ident, Span};
use proc_macro_crate::{crate_name as crate_name2, FoundCrate};

pub fn crate_name() -> Ident {
    let found_crate = crate_name2("udled").expect("udled is present in `Cargo.toml`");

    match found_crate {
        FoundCrate::Itself => {
            if !cfg!(test) {
                Ident::new("udled", Span::call_site())
            } else {
                Ident::new("crate", Span::call_site())
            }
        }
        FoundCrate::Name(name) => Ident::new(&name, Span::call_site()),
    }
}
