use crate::validator::boundary::{
    get_collection_element_type, is_collection, is_map_type, is_option_type,
};
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
    depth: u8,
    condition_fn: impl Fn(TokenStream) -> TokenStream,
) -> (TokenStream, TokenStream) {
    let name = &info.name;
    if depth > 0 && is_collection(&info.ty) {
        // 集合类型（包括 Map 和普通集合）
        generate_collection_condition(info, condition_fn)
    } else {
        // 非集合类型（普通字段）
        if info.option_ty.is_some() {
            // Option<T>
            let inner_condition = condition_fn(quote! { inner });
            let any_cond = quote! {
                self.#name.as_ref().map_or(false, |inner| #inner_condition)
            };
            let all_cond = quote! {
                self.#name.as_ref().map_or(true, |inner| !#inner_condition)
            };
            (any_cond, all_cond)
        } else {
            // 普通字段
            let self_condition = condition_fn(quote! { self.#name });
            let any_cond = quote! { #self_condition };
            let all_cond = quote! { !#self_condition };
            (any_cond, all_cond)
        }
    }
}
/// 生成集合类型的条件表达式，直接返回最终字段判定条件表达式
pub fn generate_collection_condition(
    info: &MetaInfo,
    condition_fn: impl Fn(TokenStream) -> TokenStream,
) -> (TokenStream, TokenStream) {
    let name = &info.name;
    let condition = condition_fn(quote! { v });
    let is_option = is_option_type(&info.ty);

    let element_ty = match get_collection_element_type(&info.name, &info.span, &info.ty) {
        Ok(ty) => ty,
        Err(err) => {
            let err_ts = err.to_compile_error();
            return (err_ts.clone(), err_ts);
        }
    };
    let is_element_option = is_option_type(&element_ty);

    // 顶级类型是否是 Option
    if is_option {
        // 顶级 Option 包裹
        if is_map_type(&element_ty) {
            // Map 类型
            if is_element_option {
                (
                    // any 条件：有一个 map 中 value 为 Some，且满足条件
                    quote! {
                        self.#name.as_ref()
                            .map_or(false, |map|
                                map.iter().any(|(_, value)| value.as_ref().map_or(false, |v| #condition))
                            )
                    },
                    // all 条件：map 中所有 value（None算作true）都不满足条件
                    quote! {
                        self.#name.as_ref()
                            .map_or(true, |map|
                                map.iter().all(|(_, value)| value.as_ref().map_or(true, |v| !#condition))
                            )
                    },
                )
            } else {
                (
                    quote! {
                        self.#name.as_ref()
                            .map_or(false, |map|
                                map.iter().any(|(_, value)| #condition)
                            )
                    },
                    quote! {
                        self.#name.as_ref()
                            .map_or(true, |map|
                                map.iter().all(|(_, value)| !#condition)
                            )
                    },
                )
            }
        } else {
            // 普通集合类型 (Vec/Set)
            if is_element_option {
                (
                    quote! {
                        self.#name.as_ref()
                            .map_or(false, |vec|
                                vec.iter().any(|value| value.as_ref().map_or(false, |v| #condition))
                            )
                    },
                    quote! {
                        self.#name.as_ref()
                            .map_or(true, |vec|
                                vec.iter().all(|value| value.as_ref().map_or(true, |v| !#condition))
                            )
                    },
                )
            } else {
                (
                    quote! {
                        self.#name.as_ref()
                            .map_or(false, |vec|
                                vec.iter().any(|v| #condition)
                            )
                    },
                    quote! {
                        self.#name.as_ref()
                            .map_or(true, |vec|
                                vec.iter().all(|v| !#condition)
                            )
                    },
                )
            }
        }
    } else {
        // 顶级类型不是 Option (如直接 Vec/Map)
        if is_map_type(&element_ty) {
            // Map 类型
            if is_element_option {
                (
                    quote! {
                        self.#name.iter()
                            .any(|(_, value)| value.as_ref().map_or(false, |v| #condition))
                    },
                    quote! {
                        self.#name.iter()
                            .all(|(_, value)| value.as_ref().map_or(true, |v| !#condition))
                    },
                )
            } else {
                (
                    quote! {
                        self.#name.iter()
                            .any(|(_, value)| #condition)
                    },
                    quote! {
                        self.#name.iter()
                            .all(|(_, value)| !#condition)
                    },
                )
            }
        } else {
            // 普通集合类型
            if is_element_option {
                (
                    quote! {
                        self.#name.iter()
                            .any(|v| v.as_ref().map_or(false, |x| #condition))
                    },
                    quote! {
                        self.#name.iter()
                            .all(|v| v.as_ref().map_or(true, |x| !#condition))
                    },
                )
            } else {
                (
                    quote! {
                        self.#name.iter()
                            .any(|v| #condition)
                    },
                    quote! {
                        self.#name.iter()
                            .all(|v| !#condition)
                    },
                )
            }
        }
    }
}
// 生成单个字段的完整验证
pub fn generate_single_field(info: &MetaInfo, group: Option<&syn::Expr>, depth: u8) -> TokenStream {
    let label_identifier = syn::Ident::new(
        &format!("{}_validation_deep_{}", info.name, depth),
        info.span,
    );

    let mut checks = TokenStream::new();
    checks.extend(generate_required_check(info, depth, &label_identifier));
    checks.extend(generate_not_empty_check(info, depth, &label_identifier));
    checks.extend(generate_not_blank_check(info, depth, &label_identifier));
    checks.extend(generate_no_space_check(info, depth, &label_identifier));
    checks.extend(generate_size_check(info, depth, &label_identifier));
    checks.extend(generate_range_check(info, depth, &label_identifier));
    checks.extend(generate_within_check(info, depth, &label_identifier));
    checks.extend(generate_out_of_check(info, depth, &label_identifier));
    checks.extend(generate_regex_check(info, depth, &label_identifier));
    checks.extend(generate_func_check(info, depth, &label_identifier));
    checks.extend(generate_deep_check(info, group, depth + 1));

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
