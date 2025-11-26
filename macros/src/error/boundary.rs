pub fn is_struct(ast: &syn::DeriveInput) -> bool {
    matches!(ast.data, syn::Data::Struct(_))
}

pub fn is_def_suffix(ident: &syn::Ident) -> bool {
    ident.to_string().ends_with("Def")
}

/// 得到前缀
pub fn get_base_name(ident: &syn::Ident) -> String {
    let s = ident.to_string();
    s.trim_end_matches("Def").to_string()
}

/// 如果结构体是pub，可以给出警告（可选）
pub fn warn_if_pub(vis: &syn::Visibility, ast: &syn::DeriveInput) -> Option<proc_macro2::TokenStream> {
    if matches!(vis, syn::Visibility::Public(_)) {
        Some(syn::Error::new_spanned(ast, "Struct visibility is pub, which is discouraged for error pools!").to_compile_error())
    } else {
        None
    }
}