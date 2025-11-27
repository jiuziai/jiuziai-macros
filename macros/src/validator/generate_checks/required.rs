use crate::validator::boundary::{
    get_collection_element_type, is_collection, is_map_type, is_option_type,
};
use crate::validator::generate_checks::get_validation_message;
use crate::validator::types::MetaInfo;
use proc_macro2::Ident;
use proc_macro2::TokenStream;
use quote::quote;

/// required 不参与 any/all 模式区分
pub fn generate_required_check(
    info: &MetaInfo,
    depth: u8,
    label_identifier: &Ident,
) -> TokenStream {
    let required = match &info.required {
        Some(r) => r,
        None => return quote! {},
    };

    if is_collection(&info.ty) {
        let element_ty = match get_collection_element_type(&info.name, &info.span, &info.ty) {
            Ok(ty) => ty,
            Err(e) => return e.to_compile_error(),
        };
        if !is_option_type(&element_ty) {
            return syn::Error::new(
                required.span,
                "`required` check can only be used on Option types",
            )
            .to_compile_error();
        }
    }

    let name = &info.name;
    let message =
        get_validation_message(&info.message, &required.message, "required", required.span);

    if info.message.is_some() {
        if depth > 0 && is_collection(&info.ty) {
            let element_ty = match get_collection_element_type(&info.name, &info.span, &info.ty) {
                Ok(ty) => ty,
                Err(e) => return e.to_compile_error(),
            };
            if is_option_type(&element_ty) {
                if is_map_type(&element_ty) {
                    // 集合any模式(map)
                    quote! {
                        if  #label_identifier && self.#name.as_ref().map_or(true, |(_,value)| value.iter().all(|v| v.is_some())) {
                            #label_identifier = false
                        }
                    }
                } else {
                    // 集合any模式(vec)
                    quote! {
                        if #label_identifier && self.#name.as_ref().map_or(true, |value| value.iter().all(|v| v.is_some())) {
                            #label_identifier = false
                        }
                    }
                }
            } else {
                return syn::Error::new(
                    required.span,
                    "`required` check can only be used on Option types",
                )
                .to_compile_error();
            }
        } else {
            // 非集合any模式
            quote! {
                if #label_identifier && self.#name.is_some() {
                    #label_identifier = false
                }
            }
        }
    } else {
        if depth > 0 && is_collection(&info.ty) {
            let element_ty = match get_collection_element_type(&info.name, &info.span, &info.ty) {
                Ok(ty) => ty,
                Err(e) => return e.to_compile_error(),
            };
            if is_option_type(&element_ty) {
                if is_map_type(&element_ty) {
                    // 集合all模式(map)
                    quote! {
                        if self.#name.as_ref().map_or(true, |(_,value)| value.iter().any(|v| v.is_none())) {
                            return Err(#message)
                        }
                    }
                } else {
                    // 集合all模式(vec)
                    quote! {
                        if  self.#name.as_ref().map_or(true, |value| value.iter().any(|v| v.is_none())) {
                            return Err(#message)
                        }
                    }
                }
            } else {
                return syn::Error::new(
                    required.span,
                    "`required` check can only be used on Option types",
                )
                .to_compile_error();
            }
        } else {
            // 非集合all模式
            quote! {
                if !self.#name.is_some() {
                    return Err(#message)
                }

            }
        }
    }
}
