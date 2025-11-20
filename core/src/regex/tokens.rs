use quote::{format_ident, quote};
use syn::{Item, ItemMod, Lit, LitStr};

pub fn regexes_static_gen(mut module: ItemMod) -> proc_macro2::TokenStream {
    // Ensure inline module
    let (_, items) = match module.content.take() {
        Some((brace, items)) => (brace, items),
        None => {
            return syn::Error::new_spanned(
                module,
                "regexes_static only supports inline modules (mod name { ... })",
            )
            .to_compile_error();
        }
    };

    // Collect entries: only accept const NAME: &str = "literal";
    struct Entry {
        ident: syn::Ident,
        lit: LitStr,
    }
    let mut entries: Vec<Entry> = Vec::new();

    for item in &items {
        if let Item::Const(c) = item {
            // try to extract LitStr from c.expr
            match &*c.expr {
                syn::Expr::Lit(syn::ExprLit {
                    lit: Lit::Str(s), ..
                }) => {
                    entries.push(Entry {
                        ident: c.ident.clone(),
                        lit: s.clone(),
                    });
                }
                other => {
                    return syn::Error::new_spanned(
                        other,
                        "regexes_static: const must be a string literal, e.g. `pub const NAME: &str = r\"...\";`",
                    )
                        .to_compile_error();
                }
            }
        }
    }

    if entries.is_empty() {
        // No entries: just return original module unchanged
        return quote! {
            #module
        };
    }

    // Build generated items (statics, enum variants, match arms, names, from_name arms)
    let mut static_decls = Vec::new();
    let mut match_arms = Vec::new();
    let mut name_strings = Vec::new();
    let mut from_name_arms = Vec::new();
    let mut variants = Vec::new();

    for e in &entries {
        let name = e.ident.to_string();
        // static ident: PAT_<NAME>
        let static_ident = format_ident!("PAT_{}", e.ident.to_string());
        let lit = &e.lit;

        // static Lazy<Regex>
        static_decls.push(quote! {
            #[allow(dead_code)]
            static #static_ident: ::once_cell::sync::Lazy<::regex::Regex> =
                ::once_cell::sync::Lazy::new(|| ::regex::Regex::new(#lit).unwrap());
        });

        let variant_ident = format_ident!("{}", e.ident.to_string()); // keep upper-case IDENT as variant

        match_arms.push(quote! {
            Patterns::#variant_ident => &#static_ident,
        });

        from_name_arms.push(quote! {
            #name => Some(Patterns::#variant_ident),
        });

        name_strings.push(quote! { #name });
        variants.push(quote! { #variant_ident });
    }

    let count = variants.len();

    // Reconstruct module: original items + generated content inside module body
    let mod_attrs = &module.attrs;
    let mod_vis = &module.vis;
    let mod_ident = &module.ident;

    let generated = quote! {
        // generated lazy statics
        #(#static_decls)*

        #[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
        pub enum Patterns {
            #(#variants),*
        }

        impl Patterns {
            /// Return &'static Regex (initialized on first use)
            pub fn regex(&self) -> &'static ::regex::Regex {
                match self {
                    #(#match_arms)*
                }
            }

            pub fn names() -> &'static [&'static str] {
                static NAMES: [&str; #count] = [#( #name_strings ),*];
                &NAMES
            }

            pub fn from_name(s: &str) -> Option<Self> {
                match s {
                    #(#from_name_arms)*
                    _ => None
                }
            }
        }
    };

    // Put original items back into module body and append generated
    // `items` is Vec<syn::Item> we extracted from module
    let tokens = quote! {
        #(#mod_attrs)*
        #mod_vis mod #mod_ident {
            #(#items)*
            #generated
        }
    };

    tokens
}
