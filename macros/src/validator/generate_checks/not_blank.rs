use crate::validator::boundary::{is_string_type, strip_option};
use crate::validator::generate_checks::{
    generate_option_condition, generate_validation_code, get_validation_message,
};
use crate::validator::types::MetaInfo;
use proc_macro2::{Ident, TokenStream};
use quote::quote;
use syn::Error;

pub fn generate_not_blank_check(info: &MetaInfo,is_coll:bool, label_identifier: &Ident) -> TokenStream {
    let not_blank = match &info.not_blank {
        Some(ne) => ne,
        None => return quote! {},
    };

    if !is_string_type(&strip_option(&info.ty)) {
        return Error::new(
            not_blank.span,
            "`not_blank` check can only be used on String or &str types",
        )
        .to_compile_error();
    }

    let message = get_validation_message(
        &info.message,
        &not_blank.message,
        "not_blank",
        not_blank.span,
    );

    let (any_cond, all_cond) =
        generate_option_condition(info, |var| quote! {!#var.as_str().trim().is_empty()});

    generate_validation_code(info, message, any_cond, all_cond, label_identifier)
}
