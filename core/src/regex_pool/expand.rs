use crate::regex_pool::types::FieldInfo;
use proc_macro2::Ident;
use quote::{format_ident, quote};
use syn::DeriveInput;


fn make_struct_name(base_name: &str) -> Ident {
    format_ident!("{}", base_name)
}

fn make_static_name(base_name: &str) -> Ident {
    let mut buf = String::new();
    for (i, c) in base_name.chars().enumerate() {
        if i != 0 && c.is_uppercase() {
            buf.push('_');
        }
        buf.push(c.to_ascii_uppercase());
    }
    format_ident!("{}", buf)
}
pub fn expand_regex_pool(
    _input: &DeriveInput,
    base_name: &str,
    fields: &[FieldInfo],
) -> proc_macro2::TokenStream {
    let struct_name = make_struct_name(base_name); // e.g. MyRegex
    let static_name_ident = make_static_name(base_name); // e.g. MY_REGEX

    let field_decls = fields.iter().map(|f| {
        let name = &f.name;
        quote! { pub #name: &'static ::regex::Regex, }
    });

    let field_inits = fields.iter().map(|f| {
        let name = &f.name;
        let regex = &f.regex;
        quote! {
            #name: Box::leak(Box::new(::regex::Regex::new(#regex).unwrap()))
        }
    });

    quote! {
        pub struct #struct_name {
            #(#field_decls)*
        }
        pub static #static_name_ident: ::once_cell::sync::Lazy<#struct_name> =
            ::once_cell::sync::Lazy::new(|| #struct_name {
                #(#field_inits),*
            });
    }
}