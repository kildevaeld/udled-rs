use darling::ast::NestedMeta;
use darling::{Error, FromMeta};
use proc_macro::TokenStream;
use proc_macro2::Span;
use proc_macro2::TokenStream as TokenStream2;
use quote::format_ident;
use quote::quote;
use syn::parse_macro_input;
use syn::Fields;
use syn::Ident;
use syn::ItemEnum;
use syn::LifetimeParam;

pub fn visitor(attr: TokenStream, item: TokenStream) -> TokenStream {
    let enum_item = parse_macro_input!(item as ItemEnum);

    let attr_args = match NestedMeta::parse_meta_list(attr.into()) {
        Ok(v) => v,
        Err(e) => {
            return TokenStream::from(Error::from(e).write_errors());
        }
    };

    let args = match OptionParser::from_list(&attr_args) {
        Ok(v) => v,
        Err(e) => {
            return TokenStream::from(e.write_errors());
        }
    };

    let mut output = vec![generate(&enum_item, VisitType::Ref)];

    if args.with_mut {
        output.push(generate(&enum_item, VisitType::Mut));
    }

    if args.with_fold {
        output.push(generate(&enum_item, VisitType::Fold));
    }

    quote!(
        #enum_item

        #(
            #output
        )*
    )
    .into()
}

fn fields_is_tuple(fields: &Fields) -> bool {
    fields.iter().any(|m| m.ident.is_none())
}

#[derive(FromMeta, Debug)]
struct OptionParser {
    #[darling(default)]
    with_mut: bool,
    #[darling(default)]
    with_fold: bool,
}

enum VisitType {
    Ref,
    Mut,
    Fold,
}

impl VisitType {
    fn method_name(
        &self,
        variant: impl std::fmt::Display,
        enum_name: impl std::fmt::Display,
    ) -> Ident {
        match self {
            Self::Fold => format_ident!("fold_{variant}_{enum_name}"),
            Self::Mut => format_ident!("visit_mut_{variant}_{enum_name}"),
            Self::Ref => format_ident!("visit_{variant}_{enum_name}"),
        }
    }

    fn trait_name(&self, name: impl std::fmt::Display) -> Ident {
        match self {
            Self::Fold => format_ident!("{name}Fold"),
            Self::Mut => format_ident!("{name}VisitorMut"),
            Self::Ref => format_ident!("{name}Visitor"),
        }
    }

    fn accept_name(&self) -> Ident {
        match self {
            Self::Fold => format_ident!("fold"),
            Self::Mut => format_ident!("accept_mut"),
            Self::Ref => format_ident!("accept"),
        }
    }

    fn reference(&self) -> Option<TokenStream2> {
        match self {
            Self::Fold => None,
            Self::Mut => Some(quote!(&'ast mut)),
            Self::Ref => Some(quote!(&'ast )),
        }
    }
}

fn generate(enum_item: &ItemEnum, kind: VisitType) -> TokenStream2 {
    let visitor_name = kind.trait_name(&enum_item.ident);

    let reference = kind.reference();

    let enum_name = format_ident!("{}", enum_item.ident.to_string().to_lowercase());

    let methods = enum_item.variants.iter().map(|variant| {
        let method_name = kind.method_name(variant.ident.to_string().to_lowercase(), &enum_name);

        let is_tuple = fields_is_tuple(&variant.fields);

        let fields = variant.fields.iter().map(|field| {
            let ty = &field.ty;

            if let Some(name) = &field.ident {
                quote!(
                    #name: #reference #ty
                )
            } else {
                quote!(
                    #ty
                )
            }
        });

        let fields = if is_tuple {
            if variant.fields.len() == 1 {
                quote!(
                    member: #reference #(#fields),*
                )
            } else {
                quote!(
                    member: (#(#reference #fields),*)
                )
            }
        } else {
            quote!(#(#fields),*)
        };

        quote!(
            fn #method_name(&mut self, #fields) -> Self::Output;
        )
    });

    let name = &enum_item.ident;

    let accept = enum_item.variants.iter().map(|variant| {
        let name = &variant.ident;

        let tuple = fields_is_tuple(&variant.fields);

        let fields = variant
            .fields
            .iter()
            .enumerate()
            .map(|(idx, field)| {
                if let Some(name) = &field.ident {
                    quote!(
                        #name
                    )
                } else {
                    let name = format_ident!("field_{}", idx);

                    quote!(
                        #name
                    )
                }
            })
            .collect::<Vec<_>>();

        let method_name = kind.method_name(variant.ident.to_string().to_lowercase(), &enum_name);

        if tuple {
            if fields.len() == 1 {
                quote!(
                    Self::#name(#(#fields),*) => visitor.#method_name(#(#fields),*)
                )
            } else {
                quote!(
                    Self::#name(#(#fields),*) => visitor.#method_name((#(#fields),*))
                )
            }
        } else {
            quote!(
                Self::#name { #(#fields),* } => visitor.#method_name(#(#fields),*)
            )
        }
    });

    let accept_method = kind.accept_name();

    let mut generics = enum_item.generics.clone();

    let enum_method = if reference.is_some() {
        generics
            .params
            .push(syn::GenericParam::Lifetime(LifetimeParam::new(
                syn::Lifetime::new("'ast", Span::call_site()),
            )));

        let (_generics_impl, generics_type, where_clause) = &generics.split_for_impl();

        quote!(
            pub fn #accept_method<'ast, V: #visitor_name #generics_type>(#reference self, visitor: &mut V) -> V::Output #where_clause {
                match self {
                    #(#accept),*
                }
            }
        )
    } else {
        let (_generics_impl, generics_type, where_clause) = &generics.split_for_impl();

        quote!(
                pub fn #accept_method<V: #visitor_name #generics_type>(#reference self, visitor: &mut V) -> V::Output #where_clause {
                match self {
                    #(#accept),*
                }
            }
        )
    };

    let (_generics_impl, generics_type, where_clause) = &generics.split_for_impl();

    let (enum_generics_impl, enum_generics_type, enum_where_clause) =
        &enum_item.generics.split_for_impl();

    quote!(

        pub trait #visitor_name #generics_type #where_clause {
            type Output;

            #(#methods)*
        }

        impl #enum_generics_impl  #name #enum_generics_type #enum_where_clause {
            #enum_method
        }


    )
}
