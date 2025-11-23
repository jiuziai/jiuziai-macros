use crate::validator::parser::{
    parse_bool_check, parse_func_check, parse_group_check, parse_range_check, parse_regex_check,
    parse_size_check, parse_values_check,
};
use crate::validator::types::FieldInfo;
use proc_macro2::Ident;
use syn::{Error, Type, TypePath};

pub fn check_field_info_boundary(info: &mut FieldInfo) -> syn::Result<()> {
    let has_out_message = info.message.is_some();
    // 检查内外消息边界
    if info.required.is_some()  {
        if has_out_message  {
            if  info.message.as_ref().is_some() {

            }
        }else{

        }
        if info.required.as_ref().is_none() {  }
        return Err(syn::Error::new_spanned("required", "Required"));
    }

    Ok(())
}

/// 是否是可获取长度的类型（String, &str, Vec, HashMap, BTreeMap, BTreeSet）
/// 顶层是 Option 则只看 Option 内部类型
pub fn is_len_type(ty: &Type) -> bool {
    let ty = strip_option(ty);
    match ty {
        // String 和集合
        Type::Path(TypePath { path, .. }) => {
            if let Some(seg) = path.segments.last() {
                match seg.ident.to_string().as_str() {
                    // 可取 len 的基本集合类型
                    "String" | "Vec" | "HashMap" | "BTreeMap" | "BTreeSet" => true,
                    _ => false,
                }
            } else { false }
        }
        // &str
        Type::Reference(reference) => {
            if let Type::Path(TypePath { path, .. }) = &*reference.elem {
                if let Some(seg) = path.segments.last() {
                    seg.ident == "str"
                } else { false }
            } else { false }
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
        } else { false }
    } else { false }
}

/// 判断类型是不是自定义结构体
/// 只排除常见基础类型和集合类型，Option<T> 只看 T
pub fn is_custom_struct(ty: &Type) -> bool {
    let ty = strip_option(ty); // 如果顶层是Option，则只看Option内的类型
    if let Type::Path(TypePath { path, .. }) = ty {
        if let Some(seg) = path.segments.last() {
            let name = seg.ident.to_string();
            // 非集合/基础类型就当做自定义结构体
            let basic = [
                "u8","u16","u32","u64","i8","i16","i32","i64","usize","isize",
                "f32","f64","bool","char","String","str",
                "Vec","HashMap","BTreeMap","BTreeSet"
            ];
            !basic.contains(&name.as_str())
        } else { false }
    } else { false }
}


/// helpers: Option<T> -> T
fn strip_option(ty: &Type) -> &Type {
    if let Type::Path(TypePath{ path, .. }) = ty {
        if let Some(seg) = path.segments.last() {
            if seg.ident == "Option" {
                if let syn::PathArguments::AngleBracketed(ref args) = seg.arguments {
                    for gen_arg in &args.args {
                        if let syn::GenericArgument::Type(inner_ty) = gen_arg {
                            return inner_ty;
                        }
                    }
                }
            }
        }
    }
    ty
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