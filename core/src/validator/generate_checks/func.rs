use crate::validator::generate_checks::{generate_option_condition, generate_validation_code, get_validation_message};
use crate::validator::types::MateInfo;
use proc_macro2::TokenStream;
use quote::quote;

pub fn generate_func_check(info: &MateInfo) -> TokenStream {
    let func = match &info.func {
        Some(f) => f,
        None => return quote! {},
    };

    let message = get_validation_message(
        &info.message,
        &func.message,
        "func",
        func.span,
    );

    let name = &info.name;
    let handler = &func.handler;

    let inner_condition = quote! {
        #handler(&self.#name)
    };

    let (any_cond, all_cond) = generate_option_condition(info, inner_condition);

    generate_validation_code(info, message, any_cond, all_cond)
}