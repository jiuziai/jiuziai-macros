use crate::validator::generate_checks::deep::generate_deep_check;
use crate::validator::generate_checks::func::generate_func_check;
use crate::validator::generate_checks::no_space::generate_no_space_check;
use crate::validator::generate_checks::not_blank::generate_not_blank_check;
use crate::validator::generate_checks::not_empty::generate_not_empty_check;
use crate::validator::generate_checks::out_of::generate_out_of_check;
use crate::validator::generate_checks::range::generate_range_check;
use crate::validator::generate_checks::regex::generate_regex_check;
use crate::validator::generate_checks::required::generate_required_check;
use crate::validator::generate_checks::size::generate_size_check;
use crate::validator::generate_checks::within::generate_within_check;
use crate::validator::types::MetaInfo;
use proc_macro2::{Ident, Span, TokenStream};
use quote::quote;
use syn::Error;

pub mod deep;
pub mod func;
pub mod no_space;
pub mod not_blank;
pub mod not_empty;
pub mod out_of;
pub mod range;
pub mod regex;
pub mod required;
pub mod size;
pub mod within;

// 专门处理 message 优先级和安全性的方法
pub fn get_validation_message(
    external_message: &Option<String>,
    internal_message: &Option<String>,
    check_name: &str,
    span: Span,
) -> TokenStream {
    match (external_message.as_deref(), internal_message.as_deref()) {
        (Some(ext_msg), _) => quote! { #ext_msg.to_string() },
        (None, Some(int_msg)) => quote! { #int_msg.to_string() },
        (None, None) => {
            Error::new(span, format!("{} check must have a message", check_name)).to_compile_error()
        }
    }
}

// 通用的验证代码生成器
pub fn generate_validation_code(
    info: &MetaInfo,
    message: TokenStream,
    any_condition: TokenStream,
    all_condition: TokenStream,
    label_identifier: &Ident,
) -> TokenStream {
    if info.message.is_some() {
        // any 模式：条件成立就返回成功
        quote! {
            if #label_identifier && #any_condition {
                #label_identifier = false
            }
        }
    } else {
        // all 模式：条件不成立就返回错误
        quote! {
            if #all_condition {
                return Err(#message)
            }
        }
    }
}

/// 使用示例：
/// generate_option_condition(&info, |var| quote! { #var.is_empty() })
pub fn generate_option_condition(
    info: &MetaInfo,
    condition_fn: impl Fn(TokenStream) -> TokenStream,
) -> (TokenStream, TokenStream) {
    let name = &info.name;

    if info.option_ty.is_some() {
        let inner_condition = condition_fn(quote! { inner });
        let any_cond = quote! {
            self.#name.as_ref().map_or(false, |inner| #inner_condition)
        };
        let all_cond = quote! {
            self.#name.as_ref().map_or(true, |inner| !(#inner_condition))
        };
        (any_cond, all_cond)
    } else {
        let self_condition = condition_fn(quote! { self.#name });
        let any_cond = quote! { #self_condition };
        let all_cond = quote! { !(#self_condition) };
        (any_cond, all_cond)
    }
}

// 生成单个字段的完整验证
pub fn generate_single_field(info: &MetaInfo, group: Option<&syn::Expr>, deep: u8) -> TokenStream {
    let label_identifier = syn::Ident::new(
        &format!("{}_validation_deep_{}", info.name, deep),
        info.span,
    );

    let mut checks = TokenStream::new();
    checks.extend(generate_required_check(info, &label_identifier));
    checks.extend(generate_not_empty_check(info, &label_identifier));
    checks.extend(generate_not_blank_check(info, &label_identifier));
    checks.extend(generate_no_space_check(info, &label_identifier));
    checks.extend(generate_size_check(info, &label_identifier));
    checks.extend(generate_range_check(info, &label_identifier));
    checks.extend(generate_within_check(info, &label_identifier));
    checks.extend(generate_out_of_check(info, &label_identifier));
    checks.extend(generate_regex_check(info, &label_identifier));
    checks.extend(generate_func_check(info, &label_identifier));
    checks.extend(generate_func_check(info, &label_identifier));
    checks.extend(generate_deep_check(info, group, deep + 1));

    if info.message.is_some() {
        let message = get_validation_message(
            &info.message,
            &info.message,
            info.name.to_string().as_str(),
            info.span,
        );

        quote! {
            let mut #label_identifier = true;
            #checks
            if #label_identifier {
                return Err(#message);
            }
        }
    } else {
        quote! {
            #checks
        }
    }
}
