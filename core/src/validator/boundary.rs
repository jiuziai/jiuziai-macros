use crate::validator::types::{DeepCheck, FieldInfo};
use syn::{Error, Result, Type, TypePath};

pub fn check_field_info_boundary(info: &mut FieldInfo) -> Result<()> {
    if is_option_type(&info.ty) {
        info.inner_ty = Some(strip_option(&info.ty));
    }
    is_required_type(&info)?;
    is_not_empty_type(&info)?;
    is_not_blank_type(&info)?;
    is_no_space_type(&info)?;
    is_size_type(&info)?;
    is_regex_type(&info)?;

    // 检查 deep 验证
    if let Some(deep) = &info.deep {
        is_deep_type(info)?;

        // 如果是集合类型，检查子元素类型与 deep 内部的验证规则是否兼容
        if is_collection(&info.ty) {
            if let Some(elem_ty) = get_collection_element_type(&info.ty) {
                check_deep_rules_compatibility(deep, elem_ty)?;
            }
        }
    }
    Ok(())
}
pub fn check_deep_info_boundary(info: &mut DeepCheck) -> Result<()> {
    Ok(())
}

/// 检查类型是否支持 required 检查
/// required 只允许作用于 Option 类型
pub fn is_required_type(info: &FieldInfo) -> Result<()> {
    if let Some(required) = &info.required {
        if !is_option_type(&info.ty) {
            return Err(Error::new(
                required.span,
                "`required` is only allowed to act on fields of type `Option`",
            ));
        }
    }
    Ok(())
}

/// 检查类型是否支持 not_empty 检查
/// not_empty 允许作用于字符串和集合类型
pub fn is_not_empty_type(info: &FieldInfo) -> Result<()> {
    if let Some(not_empty) = &info.not_empty {
        let check_ty = info.inner_ty.as_ref().unwrap_or(&info.ty);
        if !(is_string_type(check_ty) || is_collection(check_ty)) {
            return Err(Error::new(
                not_empty.span,
                "`not_empty` is only allowed to operate on fields of type `collection` or `string`",
            ));
        }
    }
    Ok(())
}

/// 检查类型是否支持 not_blank 检查
/// not_blank 只允许作用于字符串类型
pub fn is_not_blank_type(info: &FieldInfo) -> Result<()> {
    if let Some(not_blank) = &info.not_blank {
        let check_ty = info.inner_ty.as_ref().unwrap_or(&info.ty);
        if !is_string_type(check_ty) {
            return Err(Error::new(
                not_blank.span,
                "`not_blank` is only allowed to operate on fields of type `string`",
            ));
        }
    }
    Ok(())
}

/// 检查类型是否支持 no_space 检查
/// no_space 只允许作用于字符串类型（检查内容是否包含空格）
pub fn is_no_space_type(info: &FieldInfo) -> Result<()> {
    if let Some(no_space) = &info.no_space {
        let check_ty = info.inner_ty.as_ref().unwrap_or(&info.ty);
        if !is_string_type(check_ty) {
            return Err(Error::new(
                no_space.span,
                "`no_space` is only allowed to operate on fields of type `string`",
            ));
        }
    }
    Ok(())
}

/// 检查类型是否支持 size 检查
/// size 允许作用于字符串和集合类型（检查长度/大小）
pub fn is_size_type(info: &FieldInfo) -> Result<()> {
    if let Some(size) = &info.size {
        let check_ty = info.inner_ty.as_ref().unwrap_or(&info.ty);
        if !(is_string_type(check_ty) || is_collection(check_ty)) {
            return Err(Error::new(
                size.span,
                "`size` is only allowed to operate on fields of type `collection` or `string`",
            ));
        }
    }
    Ok(())
}

/// 检查类型是否支持 regex 检查
/// regex 允许作用于字符串类型
pub fn is_regex_type(info: &FieldInfo) -> Result<()> {
    if let Some(regex) = &info.regex {
        let check_ty = info.inner_ty.as_ref().unwrap_or(&info.ty);
        if !is_string_type(check_ty) {
            return Err(Error::new(
                regex.span,
                "`regex` is only allowed to operate on fields of type `string`",
            ));
        }
    }
    Ok(())
}

/// 检查类型是否支持 deep 检查
/// deep 不支持基础类型，其他类型都允许
pub fn is_deep_type(info: &FieldInfo) -> Result<()> {
    if let Some(deep) = &info.deep {
        let check_ty = info.inner_ty.as_ref().unwrap_or(&info.ty);

        // 只排除明确的基础类型
        let is_basic_type = if let Type::Path(TypePath { path, .. }) = check_ty {
            if let Some(seg) = path.segments.last() {
                matches!(
                    seg.ident.to_string().as_str(),
                    "u8" | "u16"
                        | "u32"
                        | "u64"
                        | "i8"
                        | "i16"
                        | "i32"
                        | "i64"
                        | "usize"
                        | "isize"
                        | "f32"
                        | "f64"
                        | "bool"
                        | "char"
                        | "String"
                        | "str"
                )
            } else {
                false
            }
        } else {
            false
        };

        if is_basic_type {
            return Err(Error::new(
                deep.span,
                "`deep` is not allowed on basic types (numbers, bool, char, string)",
            ));
        }
        // 其他类型都允许，让生成的代码检查是否实现了 Validator
    }
    Ok(())
}

/// 判断类型是不是字符串
/// Option<T> 只看 T
pub fn is_string_type(ty: &Type) -> bool {
    match ty {
        // String 类型
        Type::Path(TypePath { path, .. }) => path
            .segments
            .last()
            .map(|seg| seg.ident == "String")
            .unwrap_or(false),
        // &str
        Type::Reference(reference) => {
            if let Type::Path(TypePath { path, .. }) = &*reference.elem {
                if let Some(seg) = path.segments.last() {
                    seg.ident == "str"
                } else {
                    false
                }
            } else {
                false
            }
        }
        _ => false,
    }
}

/// 判断类型是不是集合（给定的集合类型）
/// Option<T> 只看 T
pub fn is_collection(ty: &Type) -> bool {
    let ty = strip_option(ty); // 如果顶层是Option，则只看Option内的类型
    if let Type::Path(TypePath { path, .. }) = ty {
        if let Some(seg) = path.segments.last() {
            match seg.ident.to_string().as_str() {
                "Vec" | "HashMap" | "BTreeMap" | "BTreeSet" => true,
                _ => false,
            }
        } else {
            false
        }
    } else {
        false
    }
}

/// 获取集合类型的元素类型（只取第一个类型参数）
pub fn get_collection_element_type(ty: &Type) -> Option<&Type> {
    if let Type::Path(TypePath { path, .. }) = ty {
        if let Some(seg) = path.segments.last() {
            // 只处理常见的集合类型
            match seg.ident.to_string().as_str() {
                "Vec" | "HashMap" | "BTreeMap" | "BTreeSet" | "HashSet" => {
                    if let syn::PathArguments::AngleBracketed(ref args) = seg.arguments {
                        for gen_arg in &args.args {
                            if let syn::GenericArgument::Type(inner_ty) = gen_arg {
                                return Some(inner_ty);
                            }
                        }
                    }
                }
                _ => {}
            }
        }
    }
    None
}

/// helpers: Option<T> -> T
pub fn strip_option(ty: &Type) -> Type {
    if let Type::Path(TypePath { path, .. }) = ty {
        if let Some(seg) = path.segments.last() {
            if seg.ident == "Option" {
                if let syn::PathArguments::AngleBracketed(ref args) = seg.arguments {
                    for gen_arg in &args.args {
                        if let syn::GenericArgument::Type(inner_ty) = gen_arg {
                            return inner_ty.clone();
                        }
                    }
                }
            }
        }
    }
    ty.clone()
}

/// 检查顶层类型是否是 Option
pub fn is_option_type(ty: &Type) -> bool {
    if let Type::Path(TypePath { path, .. }) = ty {
        if let Some(seg) = path.segments.last() {
            seg.ident == "Option"
        } else {
            false
        }
    } else {
        false
    }
}



/// 检查 deep 内部的验证规则与元素类型是否兼容
fn check_deep_rules_compatibility(deep: &DeepCheck, elem_ty: &Type) -> Result<()> {
    // 检查 required - 元素类型必须是 Option
    if deep.required.is_some() && !is_option_type(elem_ty) {
        return Err(Error::new(
            deep.required.as_ref().unwrap().span,
            "`required` in deep check requires the collection element type to be Option",
        ));
    }

    // 检查 not_empty/not_blank/no_space/size - 元素类型必须是字符串或集合
    if (deep.not_empty.is_some() || deep.not_blank.is_some() ||
        deep.no_space.is_some() || deep.size.is_some()) &&
        !is_size_type(elem_ty) {
        return Err(Error::new(
            deep.span,
            "`not_empty`, `not_blank`, `no_space`, `size` in deep check require string or collection element types",
        ));
    }

    // 检查 regex - 元素类型必须是字符串
    if deep.regex.is_some() && !is_string_type(elem_ty) {
        return Err(Error::new(
            deep.regex.as_ref().unwrap().span,
            "`regex` in deep check requires string element type",
        ));
    }

    // 其他检查...

    Ok(())
}
