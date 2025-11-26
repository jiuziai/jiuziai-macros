use crate::validator::boundary::{is_string_type, strip_option};
use crate::validator::generate_checks::{generate_option_condition, generate_validation_code, get_validation_message};
use crate::validator::types::MetaInfo;
use proc_macro2::{Ident, TokenStream};
use quote::quote;
use syn::Error;

pub fn generate_no_space_check(info: &MetaInfo, label_identifier: &Ident) -> TokenStream {
    let no_space = match &info.no_space {
        Some(ne) => ne,
        None => return quote! {},
    };

    if !is_string_type(&strip_option(&info.ty)) {
        return Error::new(
            no_space.span,
            "`no_space` check can only be used on String or &str types",
        )
            .to_compile_error();
    }

    let message = get_validation_message(&info.message, &no_space.message, "no_space", no_space.span);


    let (any_cond, all_cond) = generate_option_condition(info, |var|quote! {!#var.chars().any(|c| c.is_whitespace())});

    generate_validation_code(info, message, any_cond, all_cond,label_identifier)
}
