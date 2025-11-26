use crate::validator::generate_checks::{
    generate_option_condition, generate_validation_code, get_validation_message,
};
use crate::validator::types::MetaInfo;
use proc_macro2::{Ident, TokenStream};
use quote::quote;

pub fn generate_range_check(info: &MetaInfo, label_identifier: &Ident) -> TokenStream {
    let range = match &info.range {
        Some(r) => r,
        None => return quote! {},
    };

    let message = get_validation_message(&info.message, &range.message, "range", range.span);

    // 生成 min 和 max 检查
    let (any_cond, all_cond) = generate_option_condition(info, |var| {
        // 在闭包内部生成条件，这样 var 会被正确替换
        let min_check = if let Some(min_expr) = &range.min {
            quote! { #var >= #min_expr }
        } else {
            quote! { true }
        };

        let max_check = if let Some(max_expr) = &range.max {
            quote! { #var <= #max_expr }
        } else {
            quote! { true }
        };

        quote! { #min_check && #max_check }
    });

    generate_validation_code(info, message, any_cond, all_cond,label_identifier)
}
