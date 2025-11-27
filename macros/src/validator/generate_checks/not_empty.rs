use crate::validator::generate_checks::{
    generate_option_condition, generate_validation_code, get_validation_message,
};
use crate::validator::types::MetaInfo;
use proc_macro2::{Ident, TokenStream};
use quote::quote;

// 在 not_empty.rs 中
pub fn generate_not_empty_check(
    info: &MetaInfo,
    depth: u8,
    label_identifier: &Ident,
) -> TokenStream {
    let not_empty = match &info.not_empty {
        Some(ne) => ne,
        None => return quote! {},
    };

    let message = get_validation_message(
        &info.message,
        &not_empty.message,
        "not_empty",
        not_empty.span,
    );

    let (any_cond, all_cond) =
        generate_option_condition(info, depth, |var| quote! {!#var.is_empty()});

    generate_validation_code(info, message, any_cond, all_cond, label_identifier)
}
