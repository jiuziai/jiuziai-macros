extern crate proc_macro;
mod error;
mod regex_pool;
mod validator;

use crate::validator::entry::derive_validate_entry;
use proc_macro::TokenStream;
use syn::{DeriveInput, parse_macro_input};

/// 结构体验证派生宏 [派生宏](https://doc.rust-lang.org/stable/proc_macro/index.html)
#[proc_macro_derive(Validator, attributes(check))]
pub fn derive_validate(input: TokenStream) -> TokenStream {
    derive_validate_entry(parse_macro_input!(input as DeriveInput)).into()
}

/// 错误信息池懒加载预编译派生宏 [属性宏](https://doc.rust-lang.org/stable/proc_macro/index.html)
#[proc_macro_derive(Error, attributes(e))]
pub fn error_derive(input: TokenStream) -> TokenStream {
    error::entry::error_derive(input)
}

/// 正则规则池懒加载预编译派生宏 [属性宏](https://doc.rust-lang.org/stable/proc_macro/index.html)
#[proc_macro_derive(RegexPool, attributes(regex))]
pub fn regex_pool_derive(input: TokenStream) -> TokenStream {
    regex_pool::entry::regex_pool_derive(input)
}
