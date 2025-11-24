use crate::error::types::FieldInfo;

pub fn parse_fields(ast: &syn::DeriveInput) -> Result<Vec<FieldInfo>, proc_macro2::TokenStream> {
    let mut res = Vec::new();
    if let syn::Data::Struct(ref s) = ast.data {
        if let syn::Fields::Named(ref fields) = s.fields {
            for field in fields.named.iter() {
                let ident = field.ident.clone().unwrap();
                let mut code = String::new();
                let mut desc = String::new();
                for attr in &field.attrs {
                    if attr.path().is_ident("e") {
                        let r = attr.parse_nested_meta(|meta| {
                            match meta
                                .path
                                .get_ident()
                                .map(|ident| ident.to_string())
                                .as_deref()
                            {
                                Some("code") => {
                                    if let Ok(pb) = meta.value() {
                                        match pb.parse::<syn::Lit>() {
                                            Ok(syn::Lit::Str(val)) => code = val.value(),
                                            Ok(lit) => {
                                                return Err(syn::Error::new_spanned(
                                                    lit,
                                                    "code must be a string literal",
                                                ));
                                            }
                                            Err(err) => return Err(err),
                                        }
                                    }
                                }
                                Some("desc") => {
                                    if let Ok(pb) = meta.value() {
                                        match pb.parse::<syn::Lit>() {
                                            Ok(syn::Lit::Str(val)) => desc = val.value(),
                                            Ok(lit) => {
                                                return Err(syn::Error::new_spanned(
                                                    lit,
                                                    "desc must be a string literal",
                                                ));
                                            }
                                            Err(err) => return Err(err),
                                        }
                                    }
                                }
                                _ => {}
                            }
                            Ok(())
                        });
                        if let Err(e) = r {
                            return Err(e.to_compile_error());
                        }
                    }
                }

                res.push(FieldInfo { ident, code, desc });
            }
        }
    }
    Ok(res)
}
