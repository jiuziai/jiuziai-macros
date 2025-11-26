use crate::validator::boundary::{is_string_type, strip_option};
use crate::validator::generate_checks::{
    generate_option_condition, generate_validation_code, get_validation_message,
};
use crate::validator::types::MetaInfo;
use proc_macro2::{Ident, TokenStream};
use quote::quote;
use syn::Error;

pub fn generate_regex_check(info: &MetaInfo,is_coll:bool, label_identifier: &Ident) -> TokenStream {
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

    // 检查 refer 和 pattern 是否恰好只有一个存在
    match (re.refer.is_some(), re.pattern.is_some()) {
        (true, true) => {
            return Error::new(
                re.span,
                "`regex` check cannot have both `refer` and `pattern`",
            )
                .to_compile_error();
        }
        (false, false) => {
            return Error::new(
                re.span,
                "`regex` check must have either `refer` or `pattern`",
            )
                .to_compile_error();
        }
        _ => {} // 恰好有一个存在，继续执行
    }

    // 如果是 pattern，验证正则表达式语法
    if let Some(p) = &re.pattern {
        if regex::Regex::new(p).is_err() {
            return Error::new(
                re.span,
                format!("Regular expression {} for `regex` check cannot be parsed", p),
            )
                .to_compile_error();
        }
    }

    let (any_cond, all_cond) = generate_option_condition(info, |var| {
        if let Some(r) = &re.refer {
            quote! {
            #r.is_match(#var.as_str())
        }
        } else {
            let p = re.pattern.as_ref().unwrap();
            quote! {
            regex::Regex::new(#p).unwrap().is_match(#var.as_str())
        }
        }
    });

    generate_validation_code(info, message, any_cond, all_cond,label_identifier)
}
