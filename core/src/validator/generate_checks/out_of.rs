use crate::validator::generate_checks::{
    generate_option_condition, generate_validation_code, get_validation_message,
};
use crate::validator::types::MateInfo;
use proc_macro2::TokenStream;
use quote::quote;

pub fn generate_out_of_check(info: &MateInfo) -> TokenStream {
    let out_of = match &info.out_of {
        Some(r) => r,
        None => return quote! {},
    };

    let message = get_validation_message(&info.message, &out_of.message, "out_of", out_of.span);

    let name = &info.name;

    // 生成 min 和 max 检查
    let values =  &out_of.values;
    let inner_condition = quote! {
        ![#( #values ),*].contains(self.#name)
    };

    let (any_cond, all_cond) = generate_option_condition(info, inner_condition);

    generate_validation_code(info, message, any_cond, all_cond)
}
