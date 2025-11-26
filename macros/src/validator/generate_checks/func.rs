use crate::validator::boundary::is_collection;
use crate::validator::generate_checks::{
    generate_option_condition, generate_validation_code, get_validation_message,
};
use crate::validator::types::MetaInfo;
use proc_macro2::{Ident, TokenStream};
use quote::quote;

pub fn generate_func_check(
    info: &MetaInfo,
    is_coll: bool,
    label_identifier: &Ident,
) -> TokenStream {
    let func = match &info.func {
        Some(f) => f,
        None => return quote! {},
    };

    let message = get_validation_message(&info.message, &func.message, "func", func.span);

    let handler = &func.handler;

    let (any_cond, all_cond) = if is_coll {
        generate_option_condition(info, |var| quote! {#handler(&#var)})
    } else {
        generate_option_condition(info, |var| quote! {#handler(&#var)})
    };

    generate_validation_code(
        info,
        message,
        any_cond,
        all_cond,
        label_identifier,
    )
}
