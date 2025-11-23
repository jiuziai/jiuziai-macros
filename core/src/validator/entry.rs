use crate::validator::parser::parse_struct;
use proc_macro2::TokenStream;
use quote::quote;
use syn::DeriveInput;

pub fn derive_validate_entry(input: DeriveInput) -> TokenStream {
    let struct_info = match parse_struct(&input) {
        Ok(val) => val,
        Err(e) => return e.to_compile_error(),
    };

    eprintln!("Struct info: {:?}", struct_info);

    quote! {
        impl ValidateTrait for SimpleUser{
            fn check(&self) -> Result<bool, String> {
                Err(String::from("not implemented1"))
            }
            fn check_with_group(&self,gourp:&str) -> Result<bool, String> {
                Err(String::from("not implemented2"))
            }
        }
    }
}
