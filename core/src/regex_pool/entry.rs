use crate::regex_pool::{boundary, expand, parser};

use proc_macro::TokenStream;

pub fn regex_pool_derive(input: TokenStream) -> TokenStream {
    let input = syn::parse_macro_input!(input as syn::DeriveInput);

    if !boundary::is_struct(&input) {
        return syn::Error::new_spanned(&input, "Only supports structures").to_compile_error().into();
    }
    if !boundary::is_def_suffix(&input.ident) {
        return syn::Error::new_spanned(&input.ident, "The structure name must end with Def").to_compile_error().into();
    }
    let base_name = boundary::get_base_name(&input.ident);
    let mut warnings = Vec::new();
    if let Some(warning) = boundary::warn_if_pub(&input.vis, &input) {
        warnings.push(warning);
    }

    let fields = match parser::parse_fields(&input) {
        Ok(x) => x,
        Err(ts) => return ts.into(),
    };

    let expanded = expand::expand_regex_pool(&input, &base_name, &fields);

    quote::quote! {
        #(#warnings)*
        #expanded
    }.into()
}