use crate::error::types::FieldInfo;
use quote::{format_ident, quote};

/// 必须用你的外部E类型路径
const E_PATH: &str = "jiuziai_macro_libs::types::e::E";

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
        let desc_clean = desc.replace(r"\{", "{").replace(r"\}", "}");
        let template_vec: Vec<&str> = desc_clean.split("{}").collect();
        quote! {
        #ident: jiuziai_macro_libs::types::e::E {
            code: #code,
            desc: #desc_clean,
            template: &[#(#template_vec),*],
            args: Vec::new(),
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
