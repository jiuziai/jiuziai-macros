use std::os::unix::raw::ino_t;
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
use crate::validator::types::MateInfo;
use proc_macro2::{Span, TokenStream};
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
mod test;
pub mod within;

// 专门处理 message 优先级和安全性的方法
pub fn get_validation_message(
    external_message: &Option<String>,
    internal_message: &Option<String>,
    check_name: &str,
    span: Span,
) -> TokenStream {
    match (external_message.as_deref(), internal_message.as_deref()) {
        (Some(ext_msg), _) => quote! { #ext_msg },
        (None, Some(int_msg)) => quote! { #int_msg },
        (None, None) => {
            Error::new(span, format!("{} check must have a message", check_name)).to_compile_error()
        }
    }
}

// 通用的验证代码生成器
pub fn generate_validation_code(
    info: &MateInfo,
    message: TokenStream,
    any_condition: TokenStream,
    all_condition: TokenStream,
) -> TokenStream {
    if info.message.is_some() {
        // any 模式：条件成立就返回成功
        quote! {
            if #any_condition {
                return Ok(true);
            }
        }
    } else {
        // all 模式：条件不成立就返回错误
        quote! {
            if #all_condition {
                return Err(#message);
            }
        }
    }
}

/// #### *注意此处的条件 `inner_condition` 应该是 `true` 放行条件*
pub fn generate_option_condition(
    info: &MateInfo,
    inner_condition: TokenStream,
) -> (TokenStream, TokenStream) {
    let name = &info.name;

    if info.option_ty.is_some() {
        // Option 类型：先解包再应用条件
        let any_cond = quote! {
            if let Some(inner) = &self.#name {
                #inner_condition
            } else {
                true
            }
        };
        let all_cond = quote! {
            if let Some(inner) = &self.#name {
                !(#inner_condition)
            } else {
                true
            }
        };
        (any_cond, all_cond)
    } else {
        // 非 Option 类型：直接应用条件
        let any_cond = quote! { #inner_condition };
        let all_cond = quote! { !(#inner_condition) };
        (any_cond, all_cond)
    }
}

// 生成单个字段的完整验证
pub fn generate_single_field(info: &MateInfo, group: Option<&syn::Expr>) -> TokenStream {
    let mut checks = TokenStream::new();
    // 创建带后缀的变量名
    let flag_name = format!("{}_all_failed", info.name);
    let flag_ident = syn::Ident::new(&flag_name, info.name.span());
    if info.message.is_some() {
        checks.extend(quote! {
            let mut #flag_ident = false
        })
    }

    // 生成所有验证检查
    checks.extend(generate_required_check(info));
    checks.extend(generate_not_empty_check(info));
    checks.extend(generate_not_blank_check(info));
    checks.extend(generate_no_space_check(info));
    checks.extend(generate_size_check(info));
    checks.extend(generate_range_check(info));
    checks.extend(generate_within_check(info));
    checks.extend(generate_out_of_check(info));
    checks.extend(generate_regex_check(info));
    checks.extend(generate_func_check(info));
    checks.extend(generate_deep_check(info, group));

    // 模式兜底
    if info.message.is_some() {
        let message = get_validation_message(
            &info.message,
            &info.message,
            &info.name.to_string(),
            info.span,
        );
        checks.extend(quote! {
            if #flag_ident {
                return Err(#message);
            }
        })
    }

    checks
}
