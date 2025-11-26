use crate::validator::generate_checks::get_validation_message;
use crate::validator::types::MetaInfo;
use proc_macro2::Ident;
use proc_macro2::TokenStream;
use quote::quote;

/// required 不参与 any/all 模式区分
pub fn generate_required_check(info: &MetaInfo, is_coll: bool, label_identifier: &Ident) -> TokenStream {
    let required = match &info.required {
        Some(r) => r,
        None => return quote! {},
    };

    // 冗余检查：必须是 Option 类型
    if info.option_ty.is_none() {
        return syn::Error::new(
            required.span,
            "`required` check can only be used on Option types",
        )
        .to_compile_error();
    }

    let name = &info.name;
    let message =
        get_validation_message(&info.message, &required.message, "required", required.span);

    if info.message.is_some() {
        quote! {
            if #label_identifier && self.#name.is_none() {
                #label_identifier = false
            }
        }
    } else {
        quote! {
            if self.#name.is_none() {
                return Err(#message)
            }
        }
    }
}
