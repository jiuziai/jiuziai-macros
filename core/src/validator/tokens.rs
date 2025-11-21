use proc_macro2::TokenStream;
use quote::quote;
use syn::DeriveInput;

pub fn derive_validate_gen(input: DeriveInput) -> TokenStream {
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
