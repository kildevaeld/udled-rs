use proc_macro2::{Group, TokenStream};
use quote::quote;
use syn::{
    parse::{Parse, Parser},
    punctuated::Punctuated,
    Block, Expr as RustExpr, Ident, LitChar, LitStr, Token, Type,
};

mod kw {
    syn::custom_keyword!(rule);
}

pub fn parse(tokens: TokenStream) -> syn::Result<Pratt> {
    Pratt::parse.parse2(tokens)
}

pub struct Pratt {
    pub module_name: Ident,
    pub return_type: Type,
    pub rules: Vec<PrecedenceGroup>,
}

impl Parse for Pratt {
    fn parse(input: syn::parse::ParseStream) -> syn::Result<Self> {
        let module_name = input.parse::<Ident>()?;
        let _ = input.parse::<Token![->]>()?;
        let return_type = input.parse::<Type>()?;

        let mut rules = Vec::default();

        while !input.is_empty() {
            rules.push(input.parse()?);

            if input.peek(Token![-]) && input.peek2(Token![-]) {
                let _ = input.parse::<Token![-]>()?;
                let _ = input.parse::<Token![-]>()?;
            } else {
                break;
            }
        }

        Ok(Pratt {
            module_name,
            return_type,
            rules,
        })
    }
}

pub struct PrecedenceGroup {
    pub rules: Vec<Rule>,
}

impl Parse for PrecedenceGroup {
    fn parse(input: syn::parse::ParseStream) -> syn::Result<Self> {
        let mut rules = Vec::default();
        while !input.is_empty() {
            if input.peek(Token![/]) {
                let _ = input.parse::<Token![/]>()?;
            } else {
                let _ = input.parse::<kw::rule>()?;
            }

            let rule = input.parse()?;
            rules.push(rule);
            if input.peek(kw::rule) {
                continue;
            } else {
                break;
            }
        }

        Ok(PrecedenceGroup { rules })
    }
}

pub struct Rule {
    pub exprs: Vec<Expr>,
    pub action: Option<Block>,
}

impl Rule {
    pub fn peek(&self) -> TokenStream {
        let mut iter: Box<dyn Iterator<Item = _>> = if self.is_prefix() {
            Box::new(self.exprs.iter())
        } else {
            Box::new(self.exprs.iter().skip_while(|item| item.atom().is_prec()))
        };
        let first = iter.next().map(|m| m.peek()).into_iter();

        let iter = iter
            .take_while(|item| item.atom().is_token())
            .enumerate()
            .map(|(idx, item)| item.peekn(idx + 1));

        let iter = first.chain(iter).collect::<Vec<_>>();

        if iter.len() == 1 {
            quote!(
               #(#iter)&&*
            )
        } else {
            quote!(
               ( #(#iter)&&*)
            )
        }
    }

    pub fn peek_prefix(&self) -> TokenStream {
        let iter: Box<dyn Iterator<Item = _>> = if self.is_prefix() {
            Box::new(self.exprs.iter())
        } else {
            Box::new(self.exprs.iter().skip_while(|item| item.atom().is_prec()))
        };

        let iter = iter
            .take_while(|item| !item.atom().is_prec())
            .map(|item| item.peek())
            .collect::<Vec<_>>();

        if iter.len() == 1 {
            quote!(
               #(#iter)&&*
            )
        } else {
            quote!(
               ( #(#iter)&&*)
            )
        }
    }

    pub fn is_prefix(&self) -> bool {
        !self.exprs[0].atom().is_prec()
    }

    pub fn build_parse(&self, level: u8) -> TokenStream {
        let parse = self
            .exprs
            .iter()
            .skip(if self.is_prefix() { 0 } else { 1 })
            .filter_map(|expr| match expr {
                Expr::Named { name, expr } => {
                    let parse = expr.create_parse(level);
                    Some(quote!(
                        let #name = #parse;
                    ))
                }
                Expr::UnNamed { expr } => {
                    let parse = expr.create_parse(level);
                    Some(quote!(
                        let _ = #parse;
                    ))
                }
                Expr::Not { .. } => None,
            });

        let peek = self.peek();

        let first = if !self.is_prefix() {
            let first = self.exprs.first().expect("first item");
            if let Expr::Named { name, .. } = first {
                Some(quote!(
                    let #name = left;
                ))
            } else {
                None
            }
        } else {
            None
        };

        let action = if let Some(action) = &self.action {
            quote!(
                #first
                #(#parse)*
                #action
            )
        } else {
            quote!(
                {
                    (
                        #first
                        #(#parse),*
                    )
                }
            )
        };

        quote!(
            if #peek {
                #action
            }
        )
    }
}

impl Parse for Rule {
    fn parse(input: syn::parse::ParseStream) -> syn::Result<Self> {
        let mut exprs = Vec::default();

        let mut action: Option<Block> = None;

        while !input.is_empty() {
            if input.peek(syn::token::Brace) {
                action = Some(input.parse::<Block>()?);
                break;
            } else if input.peek(Token![/])
                || input.peek(kw::rule)
                || (input.peek(Token![-]) && input.peek2(Token![-]))
            {
                break;
            }

            let expr = input.parse::<Expr>()?;

            exprs.push(expr);
        }

        Ok(Rule { exprs, action })
    }
}

pub enum Expr {
    Not { expr: Atom },
    Named { name: Ident, expr: Atom },
    UnNamed { expr: Atom },
}

impl Expr {
    pub fn atom(&self) -> &Atom {
        match self {
            Expr::Named { expr, .. } => expr,
            Expr::UnNamed { expr } => expr,
            Expr::Not { expr } => expr,
        }
    }

    pub fn peek(&self) -> TokenStream {
        match self {
            Expr::Named { expr, .. } => expr.peek(),
            Expr::Not { expr } => {
                let peek = expr.peek();
                quote!(!#peek)
            }
            Expr::UnNamed { expr } => expr.peek(),
        }
    }

    pub fn peekn(&self, offset: usize) -> TokenStream {
        match self {
            Expr::Named { expr, .. } => expr.peekn(offset),
            Expr::Not { expr } => {
                let peek = expr.peekn(offset);
                quote!(!#peek)
            }
            Expr::UnNamed { expr } => expr.peekn(offset),
        }
    }
}

impl Parse for Expr {
    fn parse(input: syn::parse::ParseStream) -> syn::Result<Self> {
        let expr = if input.peek(Token![!]) {
            let _ = input.parse::<Token![!]>()?;
            Expr::Not {
                expr: input.parse()?,
            }
        } else if input.peek(Ident) && input.peek2(Token![:]) {
            let name = input.parse()?;

            let _ = input.parse::<Token![:]>()?;

            Expr::Named {
                name,
                expr: input.parse()?,
            }
        } else {
            Expr::UnNamed {
                expr: input.parse()?,
            }
        };

        Ok(expr)
    }
}

pub enum Atom {
    Prec,
    // Parser { name: syn::Type },
    Token(RustExpr),
    Rule(Vec<Rule>),
}

impl Atom {
    pub fn is_prec(&self) -> bool {
        matches!(self, Atom::Prec)
    }

    pub fn is_token(&self) -> bool {
        matches!(self, Atom::Token(_))
    }

    pub fn peek(&self) -> TokenStream {
        match self {
            Atom::Prec => panic!("cannot peek self"),
            Atom::Token(token) => quote!(input.peek(#token)?),
            Atom::Rule(rules) => {
                let iter = rules.iter().map(|item| item.peek());
                quote!(
                    #(#iter)||*
                )
            }
        }
    }

    pub fn peekn(&self, offset: usize) -> TokenStream {
        match self {
            Atom::Prec => panic!("cannot peek self"),
            Atom::Token(token) => quote!(input.peekn(#offset,#token)?),
            Atom::Rule(rules) => {
                let iter = rules.iter().map(|item| item.peek());
                quote!(
                    #(#iter)||*
                )
            }
        }
    }

    pub fn create_parse(&self, level: u8) -> TokenStream {
        match self {
            Atom::Prec => quote!(__expression(input, #level)?),
            Atom::Token(token) => quote!(input.parse(#token)?),
            Atom::Rule(rules) => {
                let iter = rules.iter().map(|item| {
                    let parse = item.build_parse(level);

                    quote!(
                        #parse
                    )
                });

                let out = quote!(
                    #(#iter)else*
                    else {
                        return Err(input.error("atom"))
                    }
                );

                out
            }
        }
    }
}

impl Parse for Atom {
    fn parse(input: syn::parse::ParseStream) -> syn::Result<Self> {
        let rule = if input.peek(Token![@]) {
            let _ = input.parse::<Token![@]>()?;
            Self::Prec
        } else if input.peek(syn::token::Paren) {
            let group = input.parse::<Group>()?;

            let rule =
                Punctuated::<Rule, Token![/]>::parse_separated_nonempty.parse2(group.stream())?; //Precedence::parse.parse2(group.stream())?;

            Self::Rule(rule.into_iter().collect())
        } else if input.peek(LitStr) || input.peek(LitChar) || input.peek(Ident) {
            Self::Token(input.parse()?)
        } else {
            return Err(input.error(format!("expected rule item: {:?}", input)));
        };

        Ok(rule)
    }
}
