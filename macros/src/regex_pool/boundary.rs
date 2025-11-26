use syn::{DeriveInput, Ident, Visibility, Data};

pub fn is_struct(input: &DeriveInput) -> bool {
    matches!(input.data, Data::Struct(_))
}

pub fn is_def_suffix(name: &Ident) -> bool {
    name.to_string().ends_with("Def")
}

pub fn get_base_name(name: &Ident) -> String {
    let s = name.to_string();
    if s.ends_with("Def") {
        s.trim_end_matches("Def").to_string()
    } else {
        s
    }
}

pub fn is_pub(vis: &Visibility) -> bool {
    matches!(vis, Visibility::Public(_))
}

pub fn warn_if_pub(vis: &Visibility, input: &DeriveInput) -> Option<proc_macro2::TokenStream> {
    if is_pub(vis) {
        Some(syn::Error::new_spanned(
            input,
            "Suggestion: Do not use pub when defining RegexPoolDef, the macro will automatically generate pub struct RegexPool and pub static REGEX_POOL."
        ).to_compile_error())
    } else {
        None
    }
}