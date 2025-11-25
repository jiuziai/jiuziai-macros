use crate::validator::boundary::{is_string_type, strip_option};
use crate::validator::generate_checks::{
    generate_option_condition, generate_validation_code, get_validation_message,
};
use crate::validator::types::MateInfo;
use proc_macro2::TokenStream;
use quote::quote;
use syn::Error;

pub fn generate_regex_check(info: &MateInfo) -> TokenStream {
    let re = match &info.regex {
        Some(ne) => ne,
        None => return quote! {},
    };

    if !is_string_type(&strip_option(&info.ty)) {
        return Error::new(
            re.span,
            "`regex` check can only be used on String or &str types",
        )
        .to_compile_error();
    }

    let message = get_validation_message(&info.message, &re.message, "regex", re.span);

    let name = &info.name;
    if re.refer.is_none() || re.pattern.is_none() {
        return Error::new(
            re.span,
            "`regex` check `refer/pattern` must have a valid value",
        )
        .to_compile_error();
    }
    let inner_condition: TokenStream;
    match (re.refer.as_ref(), re.pattern.as_ref()) {
        (Some(r), _) => {
            inner_condition = quote! {
                 #r.is_match(self.#name)
            };
        }
        (_, Some(p)) => {
            if regex::Regex::new(p).is_err() {
                return Error::new(
                    re.span,
                    format!("Regular expression {} for `regex` check cannot be parsed", p),
                )
                .to_compile_error();
            }
            inner_condition = quote! {
                regex::Regex::new(#p).unwrap().is_match(self.#name)
            };
        }
        (None, None) => {
            return Error::new(
                re.span,
                "`regex` check `refer/pattern` must have a valid value",
            )
            .to_compile_error();
        }
    }

    let (any_cond, all_cond) = generate_option_condition(info, inner_condition);

    generate_validation_code(info, message, any_cond, all_cond)
}
