use crate::validator::boundary::{get_collection_element_type, is_collection, is_string_type};
use crate::validator::generate_checks::{
    generate_option_condition, generate_validation_code, get_validation_message,
};
use crate::validator::types::{MetaInfo, MinMaxCheck};
use proc_macro2::{Ident, TokenStream};
use quote::quote;

pub fn generate_size_check(info: &MetaInfo, depth: u8, label_identifier: &Ident) -> TokenStream {
    let size = match &info.size {
        Some(s) => s,
        None => return quote! {},
    };

    let message = get_validation_message(&info.message, &size.message, "size", size.span);

    let (any_cond, all_cond) = generate_option_condition(info, depth, |var| {
        // 确定要检查的类型：如果是集合且深度>0，检查元素类型；否则检查字段本身类型
        let target_ty = if depth < 1 || !is_collection(&info.ty) {
            &info.ty
        } else {
            match get_collection_element_type(&info.name, &info.span, &info.ty) {
                Ok(element_ty) => &element_ty.clone(),
                Err(err) => return err.to_compile_error(),
            }
        };

        // 统一生成长度表达式
        let len_expr = if is_string_type(target_ty) {
            quote! { #var.chars().count() }
        } else {
            quote! { #var.len() }
        };

        generate_min_max_condition(size, len_expr)
    });

    generate_validation_code(info, message, any_cond, all_cond, label_identifier)
}

// 提取公共的 min/max 条件生成
fn generate_min_max_condition(size: &MinMaxCheck, len_expr: TokenStream) -> TokenStream {
    match (&size.min, &size.max) {
        (Some(min), Some(max)) => {
            quote! { (#len_expr >= #min && #len_expr <= #max) }
        }
        (Some(min), None) => {
            quote! { (#len_expr >= #min) }
        }
        (None, Some(max)) => {
            quote! { (#len_expr <= #max) }
        }
        _ => {
            quote! { true }
        }
    }
}
