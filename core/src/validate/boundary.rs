use crate::validate::parse::field_meta::FieldValidation;
use crate::validate::types::generic_types::GenericValidationType;

/// 验证字段规则的边界条件
pub fn validate_field_rules(validation: &FieldValidation) -> Result<(), syn::Error> {
    let validation_type = GenericValidationType::from_type(&validation.field_type);

    // 检查 not_blank 只能用于字符串类型
    if validation.not_blank.is_some() && !validation_type.is_string() {
        return Err(syn::Error::new(
            validation.ident.span(),
            "not_blank rule can only be applied to String type",
        ));
    }

    // 检查 not_empty 只能用于字符串和集合类型
    if validation.not_empty.is_some()
        && !(validation_type.is_string() || validation_type.is_collection())
    {
        return Err(syn::Error::new(
            validation.ident.span(),
            "not_empty rule can only be applied to String, Vec, HashSet, or HashMap types",
        ));
    }

    // 检查 no_space 只能用于字符串类型
    if validation.no_space.is_some() && !validation_type.is_string() {
        return Err(syn::Error::new(
            validation.ident.span(),
            "no_space rule can only be applied to String type",
        ));
    }

    // 检查 regex 只能用于字符串类型
    if validation.regex.is_some() && !validation_type.is_string() {
        return Err(syn::Error::new(
            validation.ident.span(),
            "regex rule can only be applied to String type",
        ));
    }

    // 检查 range 只能用于数值类型和时间类型
    if validation.range.is_some() && !validation_type.supports_range() {
        return Err(syn::Error::new(
            validation.ident.span(),
            "range rule can only be applied to numeric types (integers, floats, Decimal) or DateTime types",
        ));
    }

    // 检查 size 只能用于字符串和集合类型
    if validation.size.is_some() && !validation_type.supports_size() {
        return Err(syn::Error::new(
            validation.ident.span(),
            "size rule can only be applied to String, Vec, HashSet, or HashMap types",
        ));
    }

    // 检查 required 只能用于 Option 类型
    if validation.required.is_some() && !validation_type.is_option() {
        return Err(syn::Error::new(
            validation.ident.span(),
            "required rule can only be applied to Option types",
        ));
    }

    // 检查 deep 只能用于自定义结构体或包含自定义结构体的集合
    if validation.deep.is_some() {
        let supports_deep = validation_type.is_custom_struct()
            || (validation_type.is_collection() && validation_type.is_custom_struct());

        if !supports_deep {
            return Err(syn::Error::new(
                validation.ident.span(),
                "deep rule can only be applied to custom struct types or collections containing custom structs",
            ));
        }
    }

    // 检查 within 和 exclude 的类型兼容性
    // 这里简化处理，实际应该在代码生成阶段进行详细检查
    if validation.within.is_some() || validation.exclude.is_some() {
        let base_type = validation_type.get_base_type();
        if !base_type.supports_within_exclude() {
            return Err(syn::Error::new(
                validation.ident.span(),
                "within and exclude rules require compatible value types",
            ));
        }
    }

    Ok(())
}
