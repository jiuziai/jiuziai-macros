use crate::validator::boundary::{is_string_type, strip_option};
use crate::validator::generate_checks::{generate_option_condition, generate_validation_code, get_validation_message};
use crate::validator::types::MateInfo;
use proc_macro2::TokenStream;
use quote::quote;
use syn::Error;

pub fn generate_no_space_check(info: &MateInfo) -> TokenStream {
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

    let name = &info.name;
    let inner_condition = quote! {
        !self.#name.chars().any(|c| c.is_whitespace())
    };

    let (any_cond, all_cond) = generate_option_condition(info, inner_condition);

    generate_validation_code(info, message, any_cond, all_cond)
}
