use crate::validate::parse::field_meta::FieldValidation;
use quote::quote;
use syn::DeriveInput;

/// 生成 Validate trait 的实现代码

pub fn generate_validate_impl(
    input: &DeriveInput,
    fields_validation: &[FieldValidation],
) -> Result<proc_macro2::TokenStream, syn::Error> {
    let struct_name = &input.ident;

    // 生成分组类型 - 需要从所有字段的 group 属性中提取枚举类型
    let group_type = generate_group_type(fields_validation);

    // 生成 check 方法实现
    let check_impl = generate_check_impl(fields_validation);

    // 生成 check_group 方法实现
    let check_group_impl = generate_check_group_impl(fields_validation);

    let expanded = quote! {
        impl Validate for #struct_name {
            type Group = #group_type;

            fn check(&self) -> Result<bool, String> {
                #check_impl
            }

            fn check_group(&self, group: Self::Group) -> Result<bool, String> {
                #check_group_impl
            }
        }
    };

    Ok(expanded)
}

fn generate_group_type(fields_validation: &[FieldValidation]) -> proc_macro2::TokenStream {
    // 这里应该根据实际的 group 属性生成枚举类型
    // 简化处理，返回一个空枚举
    // 实际应该从所有字段的 group 属性中提取并合并枚举变体
    quote! { () }
}

fn generate_check_impl(fields_validation: &[FieldValidation]) -> proc_macro2::TokenStream {
    let field_checks: Vec<_> = fields_validation.iter().map(|field| {
        generate_field_validation_code(field, None)
    }).collect();

    quote! {
        #(#field_checks)*
        Ok(true)
    }
}

fn generate_check_group_impl(fields_validation: &[FieldValidation]) -> proc_macro2::TokenStream {
    let mut match_arms = Vec::new();

    // 收集所有唯一的分组表达式
    let mut all_groups = Vec::new();
    for field in fields_validation {
        if let Some(groups) = &field.group {
            for group_expr in groups {
                let group_str = group_expr.to_string();
                if !all_groups.iter().any(|(_, s)| s == &group_str) {
                    all_groups.push((group_expr, group_str));
                }
            }
        }
    }

    // 为每个分组生成匹配臂
    for (group_expr, _) in all_groups {
        let group_checks: Vec<_> = fields_validation.iter()
            .filter(|field| {
                field.group.as_ref().map_or(false, |groups| {
                    groups.iter().any(|g| g.to_string() == group_expr.to_string())
                })
            })
            .map(|field| generate_field_validation_code(field, Some(group_expr)))
            .collect();

        match_arms.push(quote! {
            #group_expr => {
                #(#group_checks)*
                Ok(true)
            }
        });
    }

    // 默认情况
    match_arms.push(quote! {
        _ => Ok(true)
    });

    quote! {
        match group {
            #(#match_arms),*
        }
    }
}

/// 生成单个字段的验证代码
fn generate_field_validation_code(field: &FieldValidation, group: Option<&proc_macro2::TokenStream>) -> proc_macro2::TokenStream {
    let field_ident = &field.ident;
    let mut validations = Vec::new();

    // 生成 required 验证
    if let Some(required) = &field.required {
        let message = required.message.as_ref().unwrap_or(&"字段不能为空".to_string());
        validations.push(quote! {
            if self.#field_ident.is_none() {
                return Err(#message.to_string());
            }
        });
    }

    // 生成 regex 验证
    if let Some(regex_expr) = &field.regex {
        let message = field.message.as_ref().unwrap_or(&"正则验证失败".to_string());
        validations.push(quote! {
            if let Some(value) = &self.#field_ident {
                if !#regex_expr.is_match(value) {
                    return Err(#message.to_string());
                }
            }
        });
    }

    // 生成 func 验证
    if let Some(func_options) = &field.func {
        let func_expr = &func_options.func;
        let message = func_options.message.as_ref().unwrap_or(&"函数验证失败".to_string());
        validations.push(quote! {
            if let Some(value) = &self.#field_ident {
                if !#func_expr(value) {
                    return Err(#message.to_string());
                }
            }
        });
    }

    // 生成 size 验证
    if let Some(size) = &field.size {
        if let (Some(min), Some(max)) = (size.min, size.max) {
            let message = size.message.as_ref().unwrap_or(&"大小不符合要求".to_string());
            validations.push(quote! {
                if let Some(value) = &self.#field_ident {
                    let len = value.len();
                    if len < #min || len > #max {
                        return Err(#message.to_string());
                    }
                }
            });
        }
    }

    // 生成 not_empty 验证
    if let Some(not_empty) = &field.not_empty {
        let message = not_empty.message.as_ref().unwrap_or(&"不能为空".to_string());
        validations.push(quote! {
            if let Some(value) = &self.#field_ident {
                if value.is_empty() {
                    return Err(#message.to_string());
                }
            }
        });
    }

    // 生成 within 验证
    if let Some(within) = &field.within {
        let message = within.message.as_ref().unwrap_or(&"值不在允许范围内".to_string());
        let values = &within.values;
        validations.push(quote! {
            if let Some(value) = &self.#field_ident {
                if ![#(#values),*].contains(value) {
                    return Err(#message.to_string());
                }
            }
        });
    }

    // 如果有分组信息，添加调试信息
    if let Some(group_expr) = group {
        validations.push(quote! {
            // 字段 #field_ident 在分组 #group_expr 中验证
        });
    }

    quote! {
        #(#validations)*
    }
}