use crate::validator::boundary::is_collection;
use crate::validator::generate_checks::generate_single_field;
use crate::validator::types::MetaInfo;
use proc_macro2::TokenStream;
use quote::quote;

pub fn generate_deep_check(
    info: &MetaInfo,
    group: Option<&syn::Expr>,
    deep: u8,
) -> TokenStream {
    let deep_info = match &info.deep {
        Some(d) => d,
        None => return quote! {},
    };

    let name = &deep_info.name;

    // 检查是否是集合类型
    let is_collection = is_collection(&info.ty);

    if is_collection {
        // 集合类型：使用基本校验 + 尝试特性
        // 生成基本校验
        let basic_checks = generate_single_field(&deep_info, group, deep);

        // 生成特性校验
        let trait_check: TokenStream;
        if deep_info.traits.is_some() {
            trait_check = generate_trait_check(&deep_info, group);
        } else {
            trait_check = quote! {};
        }

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
    } else {
        // 非集合类型：必须实现特性（编译报错）
        generate_trait_check(info, group)
    }
}
fn generate_trait_check(info: &MetaInfo, group: Option<&syn::Expr>) -> TokenStream {
    let field_name = &info.name;

    // 必须实现：直接调用
    if let Some(group_expr) = group {
        quote! {
            if let Some(inner_value) = &self.#field_name {
                inner_value.check_with_group(#group_expr)?;
            }
        }
    } else {
        quote! {
            if let Some(inner_value) = &self.#field_name {
                inner_value.check()?;
            }
        }
    }
}
