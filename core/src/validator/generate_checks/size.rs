use crate::validator::boundary::{is_string_type, strip_option};
use crate::validator::generate_checks::{generate_option_condition, generate_validation_code, get_validation_message};
use crate::validator::types::{MateInfo, MinMaxCheck};
use proc_macro2::TokenStream;
use quote::quote;

pub fn generate_size_check(info: &MateInfo) -> TokenStream {
    let size = match &info.size {
        Some(s) => s,
        None => return quote! {},
    };

    let message = get_validation_message(&info.message, &size.message, "size", size.span);
    let name = &info.name;

    let inner_condition = if is_string_type(&strip_option(&info.ty)) {
        // 字符串类型用 chars().count()
        let len_expr = quote! { self.#name.chars().count() };
        generate_min_max_condition(&size, len_expr)
    } else {
        // 其他类型用 .len()
        let len_expr = quote! { self.#name.len() };
        generate_min_max_condition(&size, len_expr)
    };

    let (any_cond, all_cond) = generate_option_condition(info, inner_condition);
    generate_validation_code(info, message, any_cond, all_cond)
}

// 提取公共的 min/max 条件生成
fn generate_min_max_condition(size: &MinMaxCheck, len_expr: TokenStream) -> TokenStream {
    let min_check = if let Some(min_expr) = &size.min {
        quote! { #len_expr >= #min_expr }
    } else {
        quote! { true }
    };

    let max_check = if let Some(max_expr) = &size.max {
        quote! { #len_expr <= #max_expr }
    } else {
        quote! { true }
    };

    quote! {
        #min_check && #max_check
    }
}
