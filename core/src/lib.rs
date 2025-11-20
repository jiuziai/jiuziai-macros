extern crate proc_macro;
mod regex;
mod validate;

use crate::regex::tokens::*;
use crate::validate::tokens::*;
use proc_macro::TokenStream;
use syn::{DeriveInput, ItemMod, parse_macro_input};

/// 结构体验证派生宏 [派生宏](https://doc.rust-lang.org/stable/proc_macro/index.html)
#[proc_macro_derive(
    Validate,
    attributes(
        func, not_blank, not_empty, no_space, range, regex, required, size, within, exclude, deep,
        message, group
    )
)]
pub fn derive_validate(input: TokenStream) -> TokenStream {
    let input = parse_macro_input!(input as DeriveInput);
    match derive_validate_internal(input) {
        Ok(tokens) => TokenStream::from(tokens),
        Err(err) => err.to_compile_error().into(),
    }
}

/// 正则规则懒加载编译器验证 [属性宏](https://doc.rust-lang.org/stable/proc_macro/index.html)
#[proc_macro_attribute]
pub fn regexes_static(_attr: TokenStream, item: TokenStream) -> TokenStream {
    TokenStream::from(regexes_static_gen(parse_macro_input!(item as ItemMod)))
}
