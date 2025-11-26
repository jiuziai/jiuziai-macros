use crate::validator::types::*;
use proc_macro2::Span;
use syn::{Error, Result, Type, TypePath};

pub fn check_field_info_boundary(info: &mut MetaInfo) -> Result<()> {
    check_message_rules(&info)?;
    is_required_type(&info)?;
    is_not_empty_type(&info)?;
    is_not_blank_type(&info)?;
    is_no_space_type(&info)?;
    is_size_type(&info)?;
    is_regex_type(&info)?;

    // 检查 deep 验证
    if let Some(deep) = &info.deep {
        has_deep_not_allowed(deep)?;
        check_message_rules(&deep)?;
        // 这里用父级 MateInfo 检查是否是集合
        if is_collection(&info.ty) {
            is_required_type(&deep)?;
            is_not_empty_type(&deep)?;
            is_not_blank_type(&deep)?;
            is_no_space_type(&deep)?;
            is_size_type(&deep)?;
            is_regex_type(&deep)?;
        } else {
            // 非集合类型：不允许deep有任何选项
            if has_any_validation_rules(deep) {
                return Err(Error::new(
                    deep.span,
                    "Non-collection depth verification does not allow any verification rules",
                ));
            }
        }
    }
    Ok(())
}

/// 检查 message 是否符合规则
/// 内部 message 和外部 message 不能同时存在，且不能同时不存在，要么都有内部 message，要么只有外部 message
pub fn check_message_rules(info: &MetaInfo) -> Result<()> {
    let check_single = |has_check_msg: bool, span: Span, check_name: &str| {
        let has_internal_msg = info.message.is_some();

        if has_check_msg && has_internal_msg {
            return Err(Error::new(
                span,
                format!(
                    "`{}` external message and internal message cannot exist at the same time",
                    check_name
                ),
            ));
        }
        if !has_check_msg && !has_internal_msg {
            return Err(Error::new(
                span,
                format!("`{}` at least one message must be provided", check_name),
            ));
        }
        Ok(())
    };

    if let Some(x) = &info.required {
        check_single(x.message.is_some(), x.span, "required")?;
    }
    if let Some(x) = &info.not_empty {
        check_single(x.message.is_some(), x.span, "not_empty")?;
    }
    if let Some(x) = &info.not_blank {
        check_single(x.message.is_some(), x.span, "not_blank")?;
    }
    if let Some(x) = &info.no_space {
        check_single(x.message.is_some(), x.span, "no_space")?;
    }
    if let Some(x) = &info.size {
        check_single(x.message.is_some(), x.span, "size")?;
    }
    if let Some(x) = &info.range {
        check_single(x.message.is_some(), x.span, "range")?;
    }
    if let Some(x) = &info.regex {
        check_single(x.message.is_some(), x.span, "regex")?;
    }
    if let Some(x) = &info.within {
        check_single(x.message.is_some(), x.span, "within")?;
    }
    if let Some(x) = &info.out_of {
        check_single(x.message.is_some(), x.span, "out_of")?;
    }
    if let Some(x) = &info.func {
        check_single(x.message.is_some(), x.span, "func")?;
    }
    Ok(())
}
/// 检查类型是否支持 required 检查
/// required 只允许作用于 Option 类型
pub fn is_required_type(info: &MetaInfo) -> Result<()> {
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
pub fn is_not_empty_type(info: &MetaInfo) -> Result<()> {
    if let Some(not_empty) = &info.not_empty {
        let check_ty = info.option_ty.as_ref().unwrap_or(&info.ty);
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
pub fn is_not_blank_type(info: &MetaInfo) -> Result<()> {
    if let Some(not_blank) = &info.not_blank {
        let check_ty = info.option_ty.as_ref().unwrap_or(&info.ty);
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
pub fn is_no_space_type(info: &MetaInfo) -> Result<()> {
    if let Some(no_space) = &info.no_space {
        let check_ty = info.option_ty.as_ref().unwrap_or(&info.ty);
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
pub fn is_size_type(info: &MetaInfo) -> Result<()> {
    if let Some(size) = &info.size {
        let check_ty = info.option_ty.as_ref().unwrap_or(&info.ty);
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
pub fn is_regex_type(info: &MetaInfo) -> Result<()> {
    if let Some(regex) = &info.regex {
        let check_ty = info.option_ty.as_ref().unwrap_or(&info.ty);
        if !is_string_type(check_ty) {
            return Err(Error::new(
                regex.span,
                "`regex` is only allowed to operate on fields of type `string`",
            ));
        }
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
                "Vec" | "VecDeque" | "LinkedList" | "HashSet" | "BTreeSet" | "HashMap"
                | "BTreeMap" | "SmallVec" | "IndexMap" => true,
                _ => false,
            }
        } else {
            false
        }
    } else {
        false
    }
}

/// 判断类型是不是Map（给定的集合类型）
/// Option<T> 只看 T
pub fn is_map(ty: &Type) -> bool {
    let ty = strip_option(ty); // 如果顶层是Option，则只看Option内的类型
    if let Type::Path(TypePath { path, .. }) = ty {
        if let Some(seg) = path.segments.last() {
            match seg.ident.to_string().as_str() {
                "HashMap" | "BTreeMap" | "IndexMap" => true,
                _ => false,
            }
        } else {
            false
        }
    } else {
        false
    }
}

/// 是否是字符串集合
pub fn is_string_collection(ty: &syn::Type) -> bool {
    // 检查是否是 Vec<String>, Vec<&str>, HashSet<String> 等
    let ty = strip_option(ty); // 如果顶层是Option，则只看Option内的类型
    match ty {
        syn::Type::Path(type_path) => {
            if let Some(segment) = type_path.path.segments.last() {
                if let syn::PathArguments::AngleBracketed(args) = &segment.arguments {
                    if let Some(syn::GenericArgument::Type(inner_ty)) = args.args.first() {
                        return is_string_type(inner_ty);
                    }
                }
            }
            false
        }
        _ => false,
    }
}
/// 获取集合类型的元素类型
pub fn get_collection_element_type(ty: &Type) -> Option<&Type> {
    if let Type::Path(TypePath { path, .. }) = ty {
        if let Some(seg) = path.segments.last() {
            if let syn::PathArguments::AngleBracketed(args) = &seg.arguments {
                let type_args: Vec<&Type> = args
                    .args
                    .iter()
                    .filter_map(|arg| {
                        if let syn::GenericArgument::Type(ty) = arg {
                            Some(ty)
                        } else {
                            None
                        }
                    })
                    .collect();

                match seg.ident.to_string().as_str() {
                    "Vec" | "HashSet" | "BTreeSet" if type_args.len() == 1 => {
                        Some(type_args[0]) // 元素类型
                    }
                    "HashMap" | "BTreeMap" if type_args.len() == 2 => {
                        Some(type_args[1]) // 值类型
                    }
                    _ => None,
                }
            } else {
                None
            }
        } else {
            None
        }
    } else {
        None
    }
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

/// 检查是否有任何验证规则
pub fn has_any_validation_rules(info: &MetaInfo) -> bool {
    has_valid_validation_rules(info)
        || info.message.is_some()
        || info.deep.is_some()
        || info.group.is_some()
}

pub fn has_valid_validation_rules(info: &MetaInfo) -> bool {
    info.required.is_some()
        || info.not_empty.is_some()
        || info.not_blank.is_some()
        || info.no_space.is_some()
        || info.size.is_some()
        || info.range.is_some()
        || info.within.is_some()
        || info.out_of.is_some()
        || info.regex.is_some()
        || info.func.is_some()
        || info.traits.is_some()
}

pub fn has_deep_not_allowed(info: &MetaInfo) -> Result<()> {
    if info.deep.is_some() {
        return Err(Error::new(
            info.span,
            "The `deep` attribute is not allowed in depth verification deep()",
        ));
    }
    if info.group.is_some() {
        return Err(Error::new(
            info.span,
            "The `group` attribute is not allowed in depth verification deep()",
        ));
    }
    Ok(())
}
