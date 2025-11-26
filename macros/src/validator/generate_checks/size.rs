use crate::validator::boundary::{is_string_type, strip_option};
use crate::validator::generate_checks::{generate_option_condition, generate_validation_code, get_validation_message};
use crate::validator::types::{MetaInfo, MinMaxCheck};
use proc_macro2::{Ident, TokenStream};
use quote::quote;

pub fn generate_size_check(info: &MetaInfo, label_identifier: &Ident) -> TokenStream {
    let size = match &info.size {
        Some(s) => s,
        None => return quote! {},
    };

    let message = get_validation_message(&info.message, &size.message, "size", size.span);

    let (any_cond, all_cond) = generate_option_condition(info, |var| {
        let len_expr = if is_string_type(&strip_option(&info.ty)) {
            // 字符串类型用 chars().count()
            quote! { #var.chars().count() }
        } else {
            // 其他类型用 .len()
            quote! { #var.len() }
        };

        generate_min_max_condition(size, len_expr)
    });

    generate_validation_code(info, message, any_cond, all_cond,label_identifier)
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