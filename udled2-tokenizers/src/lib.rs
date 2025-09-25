mod bool;
mod comment;
mod numeric;
mod string;

pub use self::{
    bool::Bool,
    comment::*,
    numeric::{Float, Integer},
    string::Str,
};
