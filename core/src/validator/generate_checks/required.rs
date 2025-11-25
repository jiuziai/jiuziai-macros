use crate::validator::generate_checks::get_validation_message;
use crate::validator::types::MateInfo;
use proc_macro2::TokenStream;
use quote::quote;

pub fn generate_required_check(info: &MateInfo) -> TokenStream {
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
        // any 模式：有值就返回成功
        quote! {
            if self.#name.is_some() {
                return Ok(true);
            }
        }
    } else {
        // all 模式：无值就返回错误
        quote! {
            if self.#name.is_none() {
                return Err(#message);
            }
        }
    }
}
