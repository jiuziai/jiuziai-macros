extern crate proc_macro;
mod regex;
mod validate;

use crate::regex::tokens::*;
use crate::validate::tokens::*;
use proc_macro::TokenStream;
use syn::{parse_macro_input, DeriveInput, ItemMod};

/// 结构体验证派生宏 [派生宏](https://doc.rust-lang.org/stable/proc_macro/index.html)
/// len, range, no_space, not_empty, not_blank, regex, enums, func
#[proc_macro_derive(Validate, attributes(validate))]
pub fn derive_validate(input: TokenStream) -> TokenStream {
    TokenStream::from(derive_validate_gen(parse_macro_input!(
        input as DeriveInput
    )))
}

/// 正则规则懒加载编译器验证 [属性宏](https://doc.rust-lang.org/stable/proc_macro/index.html)
#[proc_macro_attribute]
pub fn regexes_static(_attr: TokenStream, item: TokenStream) -> TokenStream {
    TokenStream::from(regexes_static_gen(parse_macro_input!(item as ItemMod)))
}
