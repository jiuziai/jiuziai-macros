//! 完整的 `#[derive(Validate)]` 派生宏实现
//!
//! 实现要点：解析 `#[validate(...)]` 属性，支持 README 中列出的校验类型，
//! 支持 any/all 模式（通过是否存在外层 `message` 判断），支持字段 `group` 属性并在 `check_group` 中按组运行，
//! 支持对 `Option<T>` 的按需校验（默认跳过 None，除非使用 `require`），以及对原始值 -> 枚举的 `TryFrom` 校验，兼容 `num_enum::FromPrimitive`。

extern crate proc_macro;
use proc_macro::TokenStream;
use quote::{quote, ToTokens};
use syn::{
    parse_macro_input, token::Comma, Data, DeriveInput, Expr, Fields, Lit, Meta,
    punctuated::Punctuated, Type, PathArguments, GenericArgument,
};
use syn::parse::Parser;

#[proc_macro_derive(Validate, attributes(validate))]
pub fn derive_validate(input: TokenStream) -> TokenStream {
    let input = parse_macro_input!(input as DeriveInput);
    let name = input.ident;

    let mut checks_tokens = Vec::new();
    let mut checks_tokens_for_group = Vec::new();

    if let Data::Struct(ds) = input.data {
        if let Fields::Named(named) = ds.fields {
            for field in named.named.into_iter() {
                let fname = field.ident.unwrap();

                // find the `validate` attribute and parse its args into NestedMeta list (optional)
                let mut found_validate: Option<Punctuated<Meta, Comma>> = None;
                for attr in field.attrs.iter() {
                    if attr.path().is_ident("validate") {
                        if let Ok(nested) = attr.parse_args_with(Punctuated::<Meta, Comma>::parse_terminated) {
                            found_validate = Some(nested);
                            break;
                        }
                    }
                }

                // build common per-field values (accessors and type analysis)
                let opt_access = quote! { &self.#fname };

                // detect Option/Vec by stringifying the type
                let ty_s = field.ty.to_token_stream().to_string();
                let is_option = ty_s.contains("Option");
                let is_vec = ty_s.contains("Vec");

                // value access: for Option we will bind `inner` and use it as value; otherwise use the field directly
                let val_access = if is_option { quote! { inner } } else { opt_access.clone() };

                // Extract inner type identifier (if Option<T> or Vec<T>) to decide whether to attempt recursive Validate calls.
                let mut inner_ident_opt: Option<String> = None;
                match &field.ty {
                    Type::Path(tp) => {
                        if let Some(seg) = tp.path.segments.last() {
                            match &seg.arguments {
                                PathArguments::AngleBracketed(ab) => {
                                    if let Some(GenericArgument::Type(inner_ty)) = ab.args.first() {
                                        if let Type::Path(itp) = inner_ty {
                                            if let Some(iseg) = itp.path.segments.last() {
                                                inner_ident_opt = Some(iseg.ident.to_string());
                                            }
                                        }
                                    }
                                }
                                _ => {
                                    inner_ident_opt = Some(seg.ident.to_string());
                                }
                            }
                        }
                    }
                    _ => {}
                }

                let try_recursive = if let Some(ref iname) = inner_ident_opt {
                    // avoid primitives and common std types
                    let primitives = ["i8","i16","i32","i64","i128","isize","u8","u16","u32","u64","u128","usize","f32","f64","bool","String","str","char"];
                    if primitives.contains(&iname.as_str()) { false } else { iname.chars().next().map(|c| c.is_uppercase()).unwrap_or(false) }
                } else { false };

                // If there is no validate attribute, we'll still consider generating a recursion-only block for nested types
                let ml_nested = found_validate.clone().unwrap_or_else(|| Punctuated::new());
                // gather top-level items
                let mut outer_message: Option<String> = None;
                let mut group_items: Vec<proc_macro2::TokenStream> = Vec::new();
                let mut checks: Vec<Meta> = Vec::new();

                for nested in ml_nested.iter() {
                    match nested {
                        Meta::NameValue(nv) if nv.path.is_ident("message") => {
                            // nv.value is Expr in syn 2
                            if let Expr::Lit(el) = &nv.value {
                                if let Lit::Str(s) = &el.lit { outer_message = Some(s.value()); }
                            }
                        }
                        Meta::List(ml2) if ml2.path.is_ident("group") => {
                            // parse inner tokens of ml2 into nested metas
                            if let Ok(inner) = Punctuated::<Meta, Comma>::parse_terminated.parse2(ml2.tokens.clone()) {
                                for g in inner.iter() {
                                    match g {
                                        Meta::Path(p) => group_items.push(p.to_token_stream()),
                                        Meta::NameValue(nv) => { if let Expr::Lit(el)=&nv.value { if let Lit::Str(s)=&el.lit { let lit = syn::LitStr::new(&s.value(), s.span()); group_items.push(quote! { #lit }); } } }
                                        _ => {}
                                    }
                                }
                            }
                        }
                        Meta::List(ml2) if ml2.path.is_ident("check") => {
                            if let Ok(inner) = Punctuated::<Meta, Comma>::parse_terminated.parse2(ml2.tokens.clone()) {
                                for c in inner.into_iter() { checks.push(c); }
                            }
                        }
                        _ => {}
                    }
                }

                        
                        // inner check generator for a single check (returns TokenStream)
                        let mut inner_checks_anymode = Vec::new();
                        let mut inner_checks_allmode = Vec::new();

                        let any_mode = outer_message.is_some();

                        for c in checks.iter() {
                            if let Meta::List(mlc) = c {
                                // parse inner tokens for this check
                                if let Ok(inner_check) = Punctuated::<Meta, Comma>::parse_terminated.parse2(mlc.tokens.clone()) {
                                    let kind = mlc.path.get_ident().map(|i| i.to_string()).unwrap_or_default();
                                    match kind.as_str() {
                                        "len" => {
                                            let mut min: Option<usize> = None; let mut max: Option<usize> = None; let mut msg: Option<String> = None;
                                            for nm in inner_check.iter() {
                                                if let Meta::NameValue(nv) = nm {
                                                    if nv.path.is_ident("min") {
                                                        if let Expr::Lit(el)=&nv.value { if let Lit::Int(li)=&el.lit{ min = li.base10_parse().ok(); } }
                                                    } else if nv.path.is_ident("max"){
                                                        if let Expr::Lit(el)=&nv.value { if let Lit::Int(li)=&el.lit{ max = li.base10_parse().ok(); } }
                                                    } else if nv.path.is_ident("message"){
                                                        if let Expr::Lit(el)=&nv.value { if let Lit::Str(s)=&el.lit{ msg = Some(s.value()); } }
                                                    }
                                                }
                                            }
                                            let msg_lit = if any_mode {
                                                let m = msg.clone().unwrap_or_default(); syn::LitStr::new(&m, proc_macro2::Span::call_site())
                                            } else {
                                                if msg.is_none() { let err = format!("missing message for 'len' check on field '{}'", stringify!(#fname)); return TokenStream::from(quote! { compile_error!(#err); }); }
                                                syn::LitStr::new(&msg.unwrap(), proc_macro2::Span::call_site())
                                            };
                                            let call = quote!{ ::jiuziai_macro_libs::validation::helpers::validate_len_str((#val_access).as_ref(), #min, #max, #msg_lit) };
                                            inner_checks_anymode.push(quote!{ match #call { Ok(_) => { passed = true; }, Err(_) => {} } });
                                            inner_checks_allmode.push(quote!{ match #call { Ok(_) => {}, Err(e) => return Err(e) } });
                                        }
                                        "range" => {
                                            let mut min: Option<i128> = None; let mut max: Option<i128> = None; let mut msg: Option<String> = None;
                                            for nm in inner_check.iter() {
                                                if let Meta::NameValue(nv) = nm {
                                                    if nv.path.is_ident("min") { if let Expr::Lit(el)=&nv.value{ if let Lit::Int(li)=&el.lit{ min = li.base10_parse().ok(); } } }
                                                    else if nv.path.is_ident("max") { if let Expr::Lit(el)=&nv.value{ if let Lit::Int(li)=&el.lit{ max = li.base10_parse().ok(); } } }
                                                    else if nv.path.is_ident("message") { if let Expr::Lit(el)=&nv.value{ if let Lit::Str(s)=&el.lit{ msg = Some(s.value()); } } }
                                                }
                                            }
                                            let msg_lit = if any_mode { let m = msg.clone().unwrap_or_default(); syn::LitStr::new(&m, proc_macro2::Span::call_site()) } else { if msg.is_none() { let err = format!("missing message for 'range' check on field '{}'", stringify!(#fname)); return TokenStream::from(quote! { compile_error!(#err); }); } syn::LitStr::new(&msg.unwrap(), proc_macro2::Span::call_site()) };
                                            let call = quote!{ ::jiuziai_macro_libs::validation::helpers::validate_range_i128((#val_access).into(), #min, #max, #msg_lit) };
                                            inner_checks_anymode.push(quote!{ match #call { Ok(_) => { passed = true; }, Err(_) => {} } });
                                            inner_checks_allmode.push(quote!{ match #call { Ok(_) => {}, Err(e) => return Err(e) } });
                                        }
                                        "size" => {
                                            let mut min: Option<usize> = None; let mut max: Option<usize> = None; let mut msg: Option<String> = None;
                                            for nm in inner_check.iter() {
                                                if let Meta::NameValue(nv) = nm {
                                                    if nv.path.is_ident("min") { if let Expr::Lit(el)=&nv.value{ if let Lit::Int(li)=&el.lit{ min = li.base10_parse().ok(); } } }
                                                    else if nv.path.is_ident("max") { if let Expr::Lit(el)=&nv.value{ if let Lit::Int(li)=&el.lit{ max = li.base10_parse().ok(); } } }
                                                    else if nv.path.is_ident("message") { if let Expr::Lit(el)=&nv.value{ if let Lit::Str(s)=&el.lit{ msg = Some(s.value()); } } }
                                                }
                                            }
                                            let msg_lit = if any_mode { let m = msg.clone().unwrap_or_default(); syn::LitStr::new(&m, proc_macro2::Span::call_site()) } else { if msg.is_none() { let err = format!("missing message for 'size' check on field '{}'", stringify!(#fname)); return TokenStream::from(quote! { compile_error!(#err); }); } syn::LitStr::new(&msg.unwrap(), proc_macro2::Span::call_site()) };
                                            let call = quote!{ ::jiuziai_macro_libs::validation::helpers::validate_size_len((#val_access).len(), #min, #max, #msg_lit) };
                                            inner_checks_anymode.push(quote!{ match #call { Ok(_) => { passed = true; }, Err(_) => {} } });
                                            inner_checks_allmode.push(quote!{ match #call { Ok(_) => {}, Err(e) => return Err(e) } });
                                        }
                                        "no_space" => {
                                            let mut msg: Option<String> = None;
                                            for nm in inner_check.iter() { if let Meta::NameValue(nv)=nm{ if nv.path.is_ident("message"){ if let Expr::Lit(el)=&nv.value{ if let Lit::Str(s)=&el.lit{ msg = Some(s.value()); } } } } }
                                            let msg_lit = if any_mode { let m = msg.clone().unwrap_or_default(); syn::LitStr::new(&m, proc_macro2::Span::call_site()) } else { if msg.is_none() { let err = format!("missing message for 'no_space' check on field '{}'", stringify!(#fname)); return TokenStream::from(quote! { compile_error!(#err); }); } syn::LitStr::new(&msg.unwrap(), proc_macro2::Span::call_site()) };
                                            let call = quote!{ ::jiuziai_macro_libs::validation::helpers::validate_no_space((#val_access).as_ref(), #msg_lit) };
                                            inner_checks_anymode.push(quote!{ match #call { Ok(_)=>{ passed = true }, Err(_) => {} } });
                                            inner_checks_allmode.push(quote!{ match #call { Ok(_)=>{}, Err(e)=> return Err(e) } });
                                        }
                                        "not_empty" => {
                                            let mut msg: Option<String> = None;
                                            for nm in inner_check.iter() { if let Meta::NameValue(nv)=nm{ if nv.path.is_ident("message"){ if let Expr::Lit(el)=&nv.value{ if let Lit::Str(s)=&el.lit{ msg = Some(s.value()); } } } } }
                                            let msg_lit = if any_mode { let m = msg.clone().unwrap_or_default(); syn::LitStr::new(&m, proc_macro2::Span::call_site()) } else { if msg.is_none() { let err = format!("missing message for 'not_empty' check on field '{}'", stringify!(#fname)); return TokenStream::from(quote! { compile_error!(#err); }); } syn::LitStr::new(&msg.unwrap(), proc_macro2::Span::call_site()) };
                                            let call = quote!{ ::jiuziai_macro_libs::validation::helpers::validate_not_empty_str((#val_access).as_ref(), #msg_lit) };
                                            inner_checks_anymode.push(quote!{ match #call { Ok(_)=>{ passed = true }, Err(_) => {} } });
                                            inner_checks_allmode.push(quote!{ match #call { Ok(_)=>{}, Err(e)=> return Err(e) } });
                                        }
                                        "not_blank" => {
                                            let mut msg: Option<String> = None;
                                            for nm in inner_check.iter() { if let Meta::NameValue(nv)=nm{ if nv.path.is_ident("message"){ if let Expr::Lit(el)=&nv.value{ if let Lit::Str(s)=&el.lit{ msg = Some(s.value()); } } } } }
                                            let msg_lit = if any_mode { let m = msg.clone().unwrap_or_default(); syn::LitStr::new(&m, proc_macro2::Span::call_site()) } else { if msg.is_none() { let err = format!("missing message for 'not_blank' check on field '{}'", stringify!(#fname)); return TokenStream::from(quote! { compile_error!(#err); }); } syn::LitStr::new(&msg.unwrap(), proc_macro2::Span::call_site()) };
                                            let call = quote!{ ::jiuziai_macro_libs::validation::helpers::validate_not_blank((#val_access).as_ref(), #msg_lit) };
                                            inner_checks_anymode.push(quote!{ match #call { Ok(_)=>{ passed = true }, Err(_) => {} } });
                                            inner_checks_allmode.push(quote!{ match #call { Ok(_)=>{}, Err(e)=> return Err(e) } });
                                        }
                                        "func" => {
                                            let mut ident: Option<String> = None; let mut msg: Option<String> = None;
                                            for nm in inner_check.iter() { if let Meta::NameValue(nv)=nm{ if nv.path.is_ident("ident"){ if let Expr::Lit(el)=&nv.value{ if let Lit::Str(s)=&el.lit{ ident = Some(s.value()); } } } else if nv.path.is_ident("message"){ if let Expr::Lit(el)=&nv.value{ if let Lit::Str(s)=&el.lit{ msg = Some(s.value()); } } } } }
                                            if let Some(id) = ident {
                                                let path: proc_macro2::TokenStream = id.parse().unwrap();
                                                let msg_lit = if any_mode { let m = msg.clone().unwrap_or_default(); syn::LitStr::new(&m, proc_macro2::Span::call_site()) } else { if msg.is_none() { let err = format!("missing message for 'func' check on field '{}'", stringify!(#fname)); return TokenStream::from(quote! { compile_error!(#err); }); } syn::LitStr::new(&msg.unwrap(), proc_macro2::Span::call_site()) };
                                                let call = quote!{ ::jiuziai_macro_libs::validation::helpers::validate_func((#val_access), #path, #msg_lit) };
                                                inner_checks_anymode.push(quote!{ match #call { Ok(_)=>{ passed = true }, Err(_) => {} } });
                                                inner_checks_allmode.push(quote!{ match #call { Ok(_)=>{}, Err(e)=> return Err(e) } });
                                            }
                                        }
                                        "regex" => {
                                            let mut pattern: Option<String> = None; let mut msg: Option<String> = None;
                                            for nm in inner_check.iter() { if let Meta::NameValue(nv)=nm{ if nv.path.is_ident("pattern"){ if let Expr::Lit(el)=&nv.value{ if let Lit::Str(s)=&el.lit{ pattern = Some(s.value()); } } } else if nv.path.is_ident("message"){ if let Expr::Lit(el)=&nv.value{ if let Lit::Str(s)=&el.lit{ msg = Some(s.value()); } } } } }
                                            if let Some(pat) = pattern {
                                                let msg_lit = if any_mode { let m = msg.clone().unwrap_or_default(); syn::LitStr::new(&m, proc_macro2::Span::call_site()) } else { if msg.is_none() { let err = format!("missing message for 'regex' check on field '{}'", stringify!(#fname)); return TokenStream::from(quote! { compile_error!(#err); }); } syn::LitStr::new(&msg.unwrap(), proc_macro2::Span::call_site()) };
                                                let call = quote!{ ::jiuziai_macro_libs::validation::helpers::validate_regex((#val_access).as_ref(), #pat, #msg_lit) };
                                                inner_checks_anymode.push(quote!{ match #call { Ok(_)=>{ passed = true }, Err(_)=>{} } });
                                                inner_checks_allmode.push(quote!{ match #call { Ok(_)=>{}, Err(e)=> return Err(e) } });
                                            }
                                        }
                                        "enums" => {
                                            // two modes: ident="TypeName" (primitive -> enum TryFrom) or list={Type::A, Type::B}
                                            let mut ident_name: Option<String> = None; let mut list_items: Vec<proc_macro2::TokenStream> = Vec::new(); let mut msg: Option<String> = None;
                                            for nm in inner_check.iter() {
                                                match nm {
                                                    Meta::NameValue(nv) if nv.path.is_ident("ident") => { if let Expr::Lit(el)=&nv.value{ if let Lit::Str(s)=&el.lit{ ident_name = Some(s.value()); } } }
                                                    Meta::List(ml3) if ml3.path.is_ident("list") => { if let Ok(inner_list) = Punctuated::<Meta, Comma>::parse_terminated.parse2(ml3.tokens.clone()) { for inner in inner_list.iter(){ if let Meta::Path(p) = inner { list_items.push(p.to_token_stream()); } } } }
                                                    Meta::NameValue(nv) if nv.path.is_ident("message") => { if let Expr::Lit(el)=&nv.value{ if let Lit::Str(s)=&el.lit{ msg = Some(s.value()); } } }
                                                    _ => {}
                                                }
                                            }
                                            let msg_lit = if any_mode { let m = msg.clone().unwrap_or_default(); syn::LitStr::new(&m, proc_macro2::Span::call_site()) } else { if msg.is_none() { let err = format!("missing message for 'enums' check on field '{}'", stringify!(#fname)); return TokenStream::from(quote! { compile_error!(#err); }); } syn::LitStr::new(&msg.unwrap(), proc_macro2::Span::call_site()) };
                                            if let Some(enum_ident) = ident_name {
                                                let enum_path: proc_macro2::TokenStream = enum_ident.parse().unwrap();
                                                let call = quote!{ ::jiuziai_macro_libs::validation::helpers::validate_enum_try_from::<#enum_path, _>((#val_access).clone(), #msg_lit) };
                                                inner_checks_anymode.push(quote!{ match #call { Ok(_)=>{ passed = true }, Err(_)=>{} } });
                                                inner_checks_allmode.push(quote!{ match #call { Ok(_)=>{}, Err(e)=> return Err(e) } });
                                            } else if !list_items.is_empty() {
                                                let list = list_items.clone();
                                                // equality compare
                                                let mut arms = Vec::new();
                                                for item in list.iter() { arms.push(quote!{ if (#val_access) == &#item { passed = true; } }); }
                                                inner_checks_anymode.push(quote!{ #(#arms)* if !passed { /* continue */ } });
                                                // allmode: if not equal to any, return Err
                                                let mut eq_conds = Vec::new();
                                                for item in list.iter() { eq_conds.push(quote!{ (#val_access) == &#item }); }
                                                let cond = quote!{ if !(#(#eq_conds)||*) { return Err(#msg_lit.to_string()); } };
                                                inner_checks_allmode.push(cond);
                                            }
                                        }
                                        "require" => {
                                            let mut msg: Option<String> = None; for nm in inner_check.iter(){ if let Meta::NameValue(nv)=nm{ if nv.path.is_ident("message"){ if let Expr::Lit(el)=&nv.value{ if let Lit::Str(s)=&el.lit{ msg = Some(s.value()); } } } } }
                                            let msg_lit = if any_mode { let m = msg.clone().unwrap_or_default(); syn::LitStr::new(&m, proc_macro2::Span::call_site()) } else { if msg.is_none() { let err = format!("missing message for 'require' check on field '{}'", stringify!(#fname)); return TokenStream::from(quote! { compile_error!(#err); }); } syn::LitStr::new(&msg.unwrap(), proc_macro2::Span::call_site()) };
                                            // applicable to Option
                                            inner_checks_anymode.push(quote!{ if (#opt_access).is_none() { /* not present -> mark not passed */ } else { passed = true; } });
                                            inner_checks_allmode.push(quote!{ if (#opt_access).is_none() { return Err(#msg_lit.to_string()); } });
                                        }
                                        _ => {}
                                    }
                                }
                            }
                        }

                        // assemble per-field block

                        // group matching tokens
                        let group_block = if group_items.is_empty() {
                            quote!{ let run_field = true; }
                        } else {
                            let group_vals = group_items.iter();
                            quote!{
                                let mut run_field = false;
                                let allowed = vec![ #( serde_json::to_value(& #group_vals ).unwrap() ),* ];
                                for av in allowed.iter() { if av == &_group { run_field = true; break; } }
                            }
                        };

                        // For check() (no grouping) we always run field; for check_group we use group_block
                        // any-mode vs all-mode

                        let any_block = if outer_message.is_some() {
                            quote!{
                                // any-mode: pass if any inner check passes
                                {
                                    let mut passed = false;
                                    #(#inner_checks_anymode)*
                                    if !passed {
                                        return Err(#outer_message.unwrap().to_string());
                                    }
                                }
                            }
                        } else {
                            quote!{
                                // all-mode: every inner check must pass
                                {
                                    #(#inner_checks_allmode)*
                                }
                            }
                        };

                        // recursion block: only attempt when the inner type appears to be a user type
                        let recursion_block = if try_recursive {
                                if is_vec {
                                if is_option {
                                    quote!{ if let Some(vec_inner) = #opt_access { for item in vec_inner.iter() { if let Err(e) = item.check() { return Err(e); } } } }
                                } else {
                                    quote!{ for item in (#val_access).iter() { if let Err(e) = item.check() { return Err(e); } } }
                                }
                            } else if is_option {
                                quote!{ if let Some(inner) = #opt_access { if let Err(e) = inner.check() { return Err(e); } } }
                            } else {
                                quote!{}
                            }
                        } else {
                            quote!{}
                        };

                        // check() version runs unconditionally
                        checks_tokens.push(quote!{
                            // field: #fname
                            #any_block
                            #recursion_block
                        });

                        // check_group version: run only if group's allowed
                        checks_tokens_for_group.push(quote!{
                            // field: #fname group filter
                            #group_block
                            if run_field {
                                #any_block
                                #recursion_block
                            }
                        });
                } // end for field
            } // end if Fields::Named
        } // end if Data::Struct

    let expanded = quote!{
        impl jiuziai_macro_libs::validation::Validate for #name {
            type Group = serde_json::Value;

            fn check(&self) -> Result<bool, String> {
                #(#checks_tokens)*
                Ok(true)
            }

            fn check_group(&self, _group: Self::Group) -> Result<bool, String> {
                let _group = _group;
                #(#checks_tokens_for_group)*
                Ok(true)
            }
        }
    };

    TokenStream::from(expanded)
}
