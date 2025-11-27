use crate::error::{boundary, expand, parser};
use proc_macro::TokenStream;

pub fn error_derive(input: TokenStream) -> TokenStream {
    let input = syn::parse_macro_input!(input as syn::DeriveInput);

    // 只允许结构体
    if !boundary::is_struct(&input) {
        return syn::Error::new_spanned(&input, "Only supports structures").to_compile_error().into();
    }

    // 结构体名必须以Def结尾
    if !boundary::is_def_suffix(&input.ident) {
        return syn::Error::new_spanned(&input.ident, "The structure name must end with Def").to_compile_error().into();
    }

    let base_name = boundary::get_base_name(&input.ident);

    let mut warnings = Vec::new();
    // 如果结构体是pub，则可以收集警告
    if let Some(warning) = boundary::warn_if_pub(&input.vis, &input) {
        warnings.push(warning);
    }

    // 字段解析（可能带属性错误）
    let fields = match parser::parse_fields(&input) {
        Ok(x) => x,
        Err(ts) => return ts.into(), // to_compile_error已处理
    };

    let expanded = expand::expand_error_pool(&base_name, &fields);

    quote::quote! {
        #(#warnings)*
        #expanded
    }.into()
}