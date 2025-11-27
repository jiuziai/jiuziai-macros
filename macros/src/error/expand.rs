use crate::error::types::FieldInfo;
use quote::{format_ident, quote};

/// 必须用你的外部E类型路径
const E_PATH: &str = "jiuziai_libs::types::e::E";

pub fn expand_error_pool(base_name: &str, fields: &[FieldInfo]) -> proc_macro2::TokenStream {
    let struct_ident = format_ident!("{}", base_name);
    let pool_ident = format_ident!("{}", base_name.to_uppercase());

    let e_typ = syn::parse_str::<syn::Type>(E_PATH).expect("Cannot parse E_TYPE");

    let struct_fields = fields.iter().map(|f| {
        let ident = &f.ident;
        quote! {

            pub #ident: #e_typ
        }
    });

    let struct_values = fields.iter().map(|f| {
        let ident = &f.ident;
        let code = &f.code;
        let desc = &f.desc;
        let template_vec: Vec<&str> = desc.split("{}").collect();
        // 处理模板数组中的转义
        let template: Vec<String> = template_vec
            .iter()
            .map(
                |s| {
                    s.replace(r"\{", "{") // 把 \{ 替换为 {
                        .replace(r"\}", "}")
                }, // 把 \} 替换为 }
            )
            .collect();
        quote! {
            #ident: jiuziai_libs::types::e::E {
                code: #code,
                desc: #desc,
                template: &[#(#template),*],
                args: Vec::new(),
                sources: Vec::new(),
            }
        }
    });

    quote! {
        pub struct #struct_ident {
            #(#struct_fields,)*
        }
        pub static #pool_ident: #struct_ident = #struct_ident {
            #(#struct_values,)*
        };
    }
}
