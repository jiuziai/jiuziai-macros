use crate::validator::boundary::{is_collection, is_string_collection, is_string_type};
use crate::validator::generate_checks::{
    generate_option_condition, generate_validation_code, get_validation_message,
};
use crate::validator::types::MetaInfo;
use proc_macro2::{Ident, TokenStream};
use quote::quote;

pub fn generate_out_of_check(info: &MetaInfo, depth: u8, label_identifier: &Ident) -> TokenStream {
    let out_of = match &info.out_of {
        Some(r) => r,
        None => return quote! {},
    };

    let message = get_validation_message(&info.message, &out_of.message, "out_of", out_of.span);
    let values = &out_of.values;

    let (any_cond, all_cond) = if is_collection(&info.ty) {
        // 集合类型：检查集合中是否包含任何排除值
        generate_option_condition(info, depth, |var| {
            if is_string_collection(&info.ty) {
                // 字符串集合：需要将元素转换为 &str 比较
                quote! { #var.iter().any(|item| matches!(item.as_str(), #( #values )|*)) }
            } else {
                // 其他类型集合：直接比较
                quote! { #var.iter().any(|item| matches!(item, #( #values )|*)) }
            }
        })
    } else if is_string_type(&info.ty) {
        // 标量字符串类型
        generate_option_condition(
            info,
            depth,
            |var| quote! {!matches!(#var.as_str(), #( #values )|*)},
        )
    } else {
        // 其他标量类型
        generate_option_condition(info, depth, |var| quote! {!matches!(#var, #( #values )|*)})
    };

    generate_validation_code(info, message, any_cond, all_cond, label_identifier)
}
