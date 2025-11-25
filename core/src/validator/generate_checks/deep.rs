use crate::validator::boundary::{
    is_collection, strip_option,
};
use crate::validator::generate_checks::generate_single_field;
use crate::validator::types::MateInfo;
use proc_macro2::{Ident, TokenStream};
use quote::quote;

pub fn generate_deep_check(info: &MateInfo, group: Option<&syn::Expr>) -> TokenStream {
    let deep_info = match &info.deep {
        Some(d) => d,
        None => return quote! {},
    };

    let name = &info.name;

    // 检查是否是集合类型
    let is_collection = is_collection(&strip_option(&info.ty));

    if is_collection {
        // 集合类型：使用基本校验 + 尝试特性
        generate_collection_deep_check(name, deep_info, group)
    } else {
        // 非集合类型：必须实现特性（编译报错）
        generate_custom_type_deep_check(name, group)
    }
}

fn generate_collection_deep_check(
    name: &Ident,
    deep_info: &MateInfo,
    group: Option<&syn::Expr>,
) -> TokenStream {
    // 生成基本校验
    let basic_checks = generate_single_field(&deep_info, None);

    // 生成特性校验
    let trait_check = generate_trait_check_for_collection(group);

    quote! {
        if let Some(inner_values) = &self.#name {
            for (index, inner_value) in inner_values.iter().enumerate() {
                // 先执行基本校验
                #basic_checks

                // 尝试特性校验
                #trait_check
            }
        }
    }
}
fn generate_custom_type_deep_check(name: &Ident, group: Option<&syn::Expr>) -> TokenStream {
    if let Some(group_expr) = group {
        quote! {
            if let Some(inner_value) = &self.#name {
                inner_value.check_with_group(#group_expr)?;
            }
        }
    } else {
        quote! {
            if let Some(inner_value) = &self.#name {
                inner_value.check()?;
            }
        }
    }
}

fn generate_trait_check_for_collection(group: Option<&syn::Expr>) -> TokenStream {
    if let Some(group_expr) = group {
        quote! {
            inner_value.check_with_group(#group_expr)?;
        }
    } else {
        quote! {
            inner_value.check()?;
        }
    }
}
