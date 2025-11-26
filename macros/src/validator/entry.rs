use crate::validator::parser::parse_struct;
use crate::validator::types::MetaInfo;
use proc_macro2::TokenStream;
use syn::DeriveInput;
use crate::validator::expand::generate_validate_impl;

pub fn derive_validate_entry(input: DeriveInput) -> TokenStream {
    let check_list: Vec<MetaInfo> = match parse_struct(&input) {
        Ok(val) => val,
        Err(e) => return e.to_compile_error(),
    };

    generate_validate_impl(&input.ident, &check_list)
}
