use crate::validator::generate_checks::{
    generate_option_condition, generate_validation_code, get_validation_message,
};
use crate::validator::types::MateInfo;
use proc_macro2::TokenStream;
use quote::quote;

pub fn generate_range_check(info: &MateInfo) -> TokenStream {
    let range = match &info.range {
        Some(r) => r,
        None => return quote! {},
    };

    let message = get_validation_message(&info.message, &range.message, "range", range.span);

    let name = &info.name;

    // 生成 min 和 max 检查
    let min_check = if let Some(min_expr) = &range.min {
        quote! { self.#name >= #min_expr }
    } else {
        quote! { true }
    };

    let max_check = if let Some(max_expr) = &range.max {
        quote! { self.#name <= #max_expr }
    } else {
        quote! { true }
    };

    let inner_condition = quote! {
        #min_check && #max_check
    };

    let (any_cond, all_cond) = generate_option_condition(info, inner_condition);

    generate_validation_code(info, message, any_cond, all_cond)
}
