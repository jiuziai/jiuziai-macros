use crate::validator::generate_checks::generate_single_field;
use crate::validator::types::*;
use proc_macro2::TokenStream;
use quote::quote;
use syn::Ident;

pub fn generate_validate_impl(struct_name: &Ident, check_list: &[MateInfo]) -> TokenStream {
    let field_checks = generate_field_checks(check_list, None);
    let group_checks = generate_group_checks(check_list);

    quote! {
        impl jiuziai_macro_libs::validate::types::Validate for #struct_name {
            pub fn check(&self) -> Result<bool, String> {
                #field_checks
                Ok(true)
            }

            pub fn check_with_group(&self, group: impl PartialEq) -> Result<bool, String> {
                #group_checks
                Ok(true)
            }
        }
    }
}

pub fn generate_field_checks(check_list: &[MateInfo], group: Option<&syn::Expr>) -> TokenStream {
    let mut checks = TokenStream::new();
    for check_info in check_list {
        checks.extend(generate_single_field(check_info, group));
    }
    checks
}

pub fn generate_group_checks(check_list: &[MateInfo]) -> TokenStream {
    // 收集所有唯一的分组表达式
    let mut all_groups = std::collections::HashSet::new();
    for field_info in check_list {
        if let Some(groups) = &field_info.group {
            for group_expr in groups {
                all_groups.insert(group_expr);
            }
        }
    }

    // 为每个分组生成 match 分支
    let group_match_arms: Vec<TokenStream> = all_groups
        .iter()
        .map(|group_expr| {
            // 过滤出属于该分组的字段
            let group_fields: Vec<MateInfo> = check_list
                .iter()
                .filter(|field_info| {
                    field_info
                        .group
                        .as_ref()
                        .map_or(false, |groups| groups.contains(group_expr))
                })
                .cloned() // 关键：克隆为 owned MateInfo
                .collect();

            // 直接使用 generate_field_checks 生成验证代码
            let group_checks = generate_field_checks(&group_fields, Some(group_expr));

            quote! {
                #group_expr => {
                    #group_checks
                    Ok(true)
                }
            }
        })
        .collect();

    quote! {
        match group {
            #(#group_match_arms,)*
            _ => self.check()
        }
    }
}
