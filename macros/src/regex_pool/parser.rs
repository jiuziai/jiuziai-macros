use crate::regex_pool::types::FieldInfo;
use syn::{DeriveInput, LitStr};

pub fn parse_fields(input: &DeriveInput) -> Result<Vec<FieldInfo>, proc_macro2::TokenStream> {
    let mut out = Vec::new();
    let fields = match &input.data {
        syn::Data::Struct(data) => &data.fields,
        _ => return Ok(out),
    };
    for field in fields {
        let name = field.ident.clone().expect("字段必须有名字");
        let mut regex = None;
        for attr in &field.attrs {
            if attr.path().is_ident("regex") {
                // 必须是 #[regex(r"...")]
                match attr.parse_args::<LitStr>() {
                    Ok(lit) => {
                        // 判定是 r"..." 原始字符串
                        let lit_str = lit.token().to_string();
                        if let Some(_s) = lit_str.strip_prefix("r") {
                            regex = Some(lit.value());
                        } else {
                            return Err(syn::Error::new_spanned(
                                attr,
                                "regex 参数必须是 Rust 原始字符串字面量，如 r\"...\""
                            ).to_compile_error());
                        }
                    }
                    Err(_) => {
                        return Err(syn::Error::new_spanned(
                            attr,
                            "regex 属性格式错误。必须类似 #[regex(r\"...\")]"
                        ).to_compile_error());
                    }
                }
            }
        }
        let regex = match regex {
            Some(r) => r,
            None => return Err(syn::Error::new_spanned(field, "字段必须有 #[regex(r\"...\")] 属性").to_compile_error()),
        };
        out.push(FieldInfo { name, regex });
    }
    Ok(out)
}