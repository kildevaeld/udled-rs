#![no_std]

extern crate alloc;

mod bool;
mod comment;
mod ident;
mod numeric;
mod string;
mod ws;

pub use self::{
    bool::Bool,
    comment::*,
    ident::*,
    numeric::{Float, Integer},
    string::Str,
    ws::*,
};
