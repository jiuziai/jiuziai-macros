use proc_macro2::TokenStream;
use syn::DeriveInput;
use crate::validate::boundary::validate_field_rules;
use crate::validate::codegen::generate_validate_impl;
use crate::validate::parse::attributes::parse_struct_attributes;

pub fn derive_validate_internal(input: DeriveInput) -> Result<TokenStream, syn::Error> {
    // 1. 解析结构体字段和属性
    let fields_validation = parse_struct_attributes(&input)?;

    // 2. 对每个字段进行边界检查
    for field_validation in &fields_validation {
        validate_field_rules(field_validation)?;
    }

    // 3. 生成验证代码
    let generated_code = generate_validate_impl(&input, &fields_validation)?;

    Ok(generated_code)
}