use crate::validator::generate_checks::{generate_option_condition, generate_validation_code, get_validation_message};
use crate::validator::types::MateInfo;
use proc_macro2::TokenStream;
use quote::quote;

pub fn generate_within_check(info: &MateInfo) -> TokenStream {
    let within = match &info.within {
        Some(r) => r,
        None => return quote! {},
    };

    let message = get_validation_message(&info.message, &within.message, "within", within.span);

    let name = &info.name;

    // 生成 min 和 max 检查
    let values =  &within.values;
    let inner_condition = quote! {
        [#( #values ),*].contains(self.#name)
    };

    let (any_cond, all_cond) = generate_option_condition(info, inner_condition);

    generate_validation_code(info, message, any_cond, all_cond)
}
