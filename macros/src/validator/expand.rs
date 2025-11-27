use crate::validator::generate_checks::generate_single_field;
use crate::validator::types::*;
use proc_macro2::TokenStream;
use quote::quote;
use syn::Ident;

pub fn generate_validate_impl(struct_name: &Ident, check_list: &[MetaInfo]) -> TokenStream {
    let field_checks = generate_field_checks(check_list, None);
    let group_checks = generate_group_checks(check_list);

    quote! {
        impl jiuziai_libs::validate::types::Validate for #struct_name {
            fn check(&self) -> Result<bool, String> {
                #field_checks
                Ok(true)
            }

            fn check_with_group(&self, group: &dyn std::any::Any) -> Result<bool, String> {
                #group_checks
            }
        }
    }
}

pub fn generate_field_checks(check_list: &[MetaInfo], group: Option<&syn::Expr>) -> TokenStream {
    let mut checks = TokenStream::new();
    for check_info in check_list {
        checks.extend(generate_single_field(check_info, group,0));
    }
    checks
}

pub fn generate_group_checks(check_list: &[MetaInfo]) -> TokenStream {
    let mut all_groups = Vec::new();
    for field_info in check_list {
        if let Some(groups) = &field_info.group {
            for group_expr in groups {
                if !all_groups.contains(group_expr) {
                    all_groups.push(group_expr.clone());
                }
            }
        }
    }

    let group_match_arms: Vec<TokenStream> = all_groups
        .iter()
        .map(|group_expr| {
            let group_fields: Vec<MetaInfo> = check_list
                .iter()
                .filter(|field_info| {
                    field_info
                        .group
                        .as_ref()
                        .map_or(false, |groups| groups.contains(group_expr))
                })
                .cloned()
                .collect();

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
        if let Some(user_group) = group.downcast_ref::<UserGroup>() {
            match user_group {
                #(#group_match_arms,)*
                _ => self.check()
            }
        } else {
            self.check()
        }
    }
}
