use crate::validator::boundary::{
    is_collection, is_string_collection, is_string_type, strip_option,
};
use crate::validator::generate_checks::{
    generate_option_condition, generate_validation_code, get_validation_message,
};
use crate::validator::types::MetaInfo;
use proc_macro2::{Ident, TokenStream};
use quote::quote;

pub fn generate_within_check(
    info: &MetaInfo,
    depth: u8,
    label_identifier: &Ident,
) -> TokenStream {
    let within = match &info.within {
        Some(r) => r,
        None => return quote! {},
    };

    let message = get_validation_message(&info.message, &within.message, "within", within.span);
    let values = &within.values;

    let (any_cond, all_cond) = if is_collection(&strip_option(&info.ty)) {
        // 集合类型：检查集合中所有元素都在允许值范围内
        generate_option_condition(info, depth, |var| {
            if is_string_collection(&info.ty) {
                quote! { #var.iter().all(|item| matches!(item.as_str(), #( #values )|*)) }
            } else {
                quote! { #var.iter().all(|item| matches!(item, #( #values )|*)) }
            }
        })
    } else if is_string_type(&info.ty) {
        // 标量字符串类型
        generate_option_condition(
            info,
            depth,
            |var| quote! {matches!(#var.as_str(), #( #values )|*)},
        )
    } else {
        // 其他标量类型
        generate_option_condition(info, depth, |var| quote! {matches!(#var, #( #values )|*)})
    };

    generate_validation_code(info, message, any_cond, all_cond, label_identifier)
}
