use crate::validator::boundary::check_field_info_boundary;
use crate::validator::types::*;
use syn::{
    Data, DataStruct, DeriveInput, Error, Expr, Field, Fields, Lit, LitStr, Meta, Result, Token,
    punctuated::Punctuated, spanned::Spanned,
};

/// 解析整个 struct 输入，返回所有带 check 的字段（FieldInfo 列表）
pub fn parse_struct(input: &DeriveInput) -> Result<Vec<FieldInfo>> {
    let mut results = Vec::new();
    match &input.data {
        Data::Struct(DataStruct {
            fields: Fields::Named(named),
            ..
        }) => {
            for field in &named.named {
                // parse_field 每个字段（如果 Field 无 check，也会返回默认的 FieldInfo，通常你可以过滤掉）
                let info = parse_field(field)?;
                // 你可以选择只 push 有 check 属性的字段
                let has_check = field.attrs.iter().any(|attr| attr.path().is_ident("check"));
                if has_check {
                    results.push(info);
                }
            }
            Ok(results)
        }
        _ => Err(Error::new(
            input.span(),
            "Only #[derive(Validator)] on structs with named fields is supported",
        )),
    }
}

/// 解析一个 struct 字段上的 FieldInfo
pub fn parse_field(field: &Field) -> Result<FieldInfo> {
    let name = field
        .ident
        .clone()
        .ok_or_else(|| Error::new(field.span(), "Field must have ident"))?;
    let ty = field.ty.clone();
    let mut info = FieldInfo::new(name, ty);
    info.span = Some(field.span());
    for attr in &field.attrs {
        if attr.path().is_ident("check") {
            let metas: Punctuated<Meta, Token![,]> =
                attr.parse_args_with(Punctuated::<Meta, Token![,]>::parse_terminated)?;
            for meta in &metas {
                parse_check_meta(meta, &mut info)?;
            }
        }
    }
    Ok(info)
}

/// 递归解析 #[check(...)] 每个 Meta
pub fn parse_check_meta(meta: &Meta, info: &mut FieldInfo) -> Result<()> {
    match meta {
        Meta::Path(path) => {
            if let Some(ident) = path.get_ident() {
                if ident == "deep" {
                    info.deep = Some(EmptyCheck {
                        value: true,
                        span: Some(path.span()),
                    });
                } else {
                    return Err(Error::new(
                        path.span(),
                        format!("Unknown unary check: {}", ident),
                    ));
                }
            } else {
                return Err(Error::new(path.span(), "Expected identifier in Meta::Path"));
            }
        }
        Meta::NameValue(nv) => {
            let ident = nv
                .path
                .get_ident()
                .ok_or_else(|| Error::new(nv.path.span(), "Expected identifier"))?;
            // 顶层只允许 message, 其它参数请用 MetaList
            if ident == "message" {
                if let Expr::Lit(expr_lit) = &nv.value {
                    if let Lit::Str(s) = &expr_lit.lit {
                        info.message = Some(s.value());
                    } else {
                        return Err(Error::new(
                            expr_lit.lit.span(),
                            "message must be string literal",
                        ));
                    }
                } else {
                    return Err(Error::new(
                        nv.value.span(),
                        "message must be string literal (Expr::Lit)",
                    ));
                }
            } else {
                return Err(Error::new(
                    nv.path.span(),
                    format!("Unknown top-level name-value check: {ident}"),
                ));
            }
        }
        Meta::List(list) => {
            let group_name = list
                .path
                .get_ident()
                .ok_or_else(|| Error::new(list.path.span(), "Expected identifier"))?;
            let args: Punctuated<Meta, Token![,]> =
                list.parse_args_with(Punctuated::<Meta, Token![,]>::parse_terminated)?;
            match group_name.to_string().as_str() {
                "required" => info.required = Some(parse_bool_check(&args)?),
                "not_empty" => info.not_empty = Some(parse_bool_check(&args)?),
                "not_blank" => info.not_blank = Some(parse_bool_check(&args)?),
                "no_space" => info.no_space = Some(parse_bool_check(&args)?),
                "size" => info.size = Some(parse_size_check(&args)?),
                "range" => info.range = Some(parse_range_check(&args)?),
                "within" => info.within = Some(parse_values_check(&args)?),
                "out_of" => info.out_of = Some(parse_values_check(&args)?),
                "regex" => info.regex = Some(parse_regex_check(&args)?),
                "func" => info.func = Some(parse_func_check(&args)?),
                "group" => info.group = parse_group_check(&args)?,
                _ => {
                    return Err(Error::new(
                        group_name.span(),
                        format!("Unknown check group: {}", group_name),
                    ));
                }
            }
        }
    }
    check_field_info_boundary(info)
}

// BoolCheck
pub fn parse_bool_check(args: &Punctuated<Meta, Token![,]>) -> Result<BoolCheck> {
    let mut message = None;
    for meta in args {
        if let Meta::NameValue(nv) = meta {
            if nv.path.is_ident("message") {
                if let Expr::Lit(expr_lit) = &nv.value {
                    if let Lit::Str(s) = &expr_lit.lit {
                        message = Some(s.value());
                    }
                }
            }
        }
    }
    Ok(BoolCheck {
        value: true,
        message,
        span: Some(args.span()),
    })
}

// SizeCheck
pub fn parse_size_check(args: &Punctuated<Meta, Token![,]>) -> Result<SizeCheck> {
    let mut min = None;
    let mut max = None;
    let mut message = None;
    for meta in args {
        if let Meta::NameValue(nv) = meta {
            if nv.path.is_ident("min") {
                if let Expr::Lit(expr_lit) = &nv.value {
                    if let Lit::Int(val) = &expr_lit.lit {
                        min = Some(val.base10_parse::<u64>()?);
                    }
                }
            } else if nv.path.is_ident("max") {
                if let Expr::Lit(expr_lit) = &nv.value {
                    if let Lit::Int(val) = &expr_lit.lit {
                        max = Some(val.base10_parse::<u64>()?);
                    }
                }
            } else if nv.path.is_ident("message") {
                if let Expr::Lit(expr_lit) = &nv.value {
                    if let Lit::Str(s) = &expr_lit.lit {
                        message = Some(s.value());
                    }
                }
            }
        }
    }
    if min.is_none() && max.is_none() {
        return Err(Error::new(
            args.span(),
            "min and max cannot be empty at the same time",
        ));
    }
    Ok(SizeCheck {
        min,
        max,
        message,
        span: Some(args.span()),
    })
}

// RangeCheck
pub fn parse_range_check(args: &Punctuated<Meta, Token![,]>) -> Result<RangeCheck> {
    let mut min = None;
    let mut max = None;
    let mut message = None;
    for meta in args {
        if let Meta::NameValue(nv) = meta {
            if nv.path.is_ident("min") {
                min = Some(nv.value.clone());
            } else if nv.path.is_ident("max") {
                max = Some(nv.value.clone());
            } else if nv.path.is_ident("message") {
                if let Expr::Lit(expr_lit) = &nv.value {
                    if let Lit::Str(s) = &expr_lit.lit {
                        message = Some(s.value());
                    }
                }
            }
        }
    }
    if min.is_none() && max.is_none() {
        return Err(Error::new(
            args.span(),
            "min and max cannot be empty at the same time",
        ));
    }
    Ok(RangeCheck {
        min,
        max,
        message,
        span: Some(args.span()),
    })
}

// ValuesCheck
pub fn parse_values_check(args: &Punctuated<Meta, Token![,]>) -> Result<ValuesCheck> {
    let mut values: Vec<Expr> = vec![];
    let mut message = None;
    for meta in args {
        match meta {
            Meta::List(list) if list.path.is_ident("values") => {
                let exprs =
                    list.parse_args_with(Punctuated::<Expr, Token![,]>::parse_terminated)?;
                for expr in exprs {
                    values.push(expr);
                }
            }
            Meta::NameValue(nv) if nv.path.is_ident("message") => {
                if let Expr::Lit(expr_lit) = &nv.value {
                    if let Lit::Str(s) = &expr_lit.lit {
                        message = Some(s.value());
                    }
                }
            }
            _ => {}
        }
    }
    if values.is_empty() {
        return Err(Error::new(args.span(), "values() must not be empty"));
    }
    Ok(ValuesCheck {
        values,
        message,
        span: Some(args.span()),
    })
}

// RegexCheck
pub fn parse_regex_check(args: &Punctuated<Meta, Token![,]>) -> Result<RegexCheck> {
    let mut refer = None;
    let mut pattern = None;
    let mut message = None;
    for meta in args {
        if let Meta::NameValue(nv) = meta {
            if nv.path.is_ident("refer") {
                refer = Some(nv.value.clone());
            } else if nv.path.is_ident("pattern") {
                if let Expr::Lit(expr_lit) = &nv.value {
                    if let Lit::Str(s) = &expr_lit.lit {
                        pattern = Some(s.value());
                    }
                }
            } else if nv.path.is_ident("message") {
                if let Expr::Lit(expr_lit) = &nv.value {
                    if let Lit::Str(s) = &expr_lit.lit {
                        message = Some(s.value());
                    }
                }
            }
        }
    }
    if refer.is_none() && pattern.is_none() {
        return Err(Error::new(
            args.span(),
            "refer and pattern cannot be empty at the same time",
        ));
    }
    Ok(RegexCheck {
        refer,
        pattern,
        message,
        span: Some(args.span()),
    })
}

// FuncCheck
pub fn parse_func_check(args: &Punctuated<Meta, Token![,]>) -> Result<FuncCheck> {
    let mut refer = None;
    let mut path = None;
    let mut message = None;
    for meta in args {
        if let Meta::NameValue(nv) = meta {
            if nv.path.is_ident("refer") {
                refer = Some(nv.value.clone());
            } else if nv.path.is_ident("path") {
                if let Expr::Path(expr_path) = &nv.value {
                    path = Some(expr_path.path.clone());
                }
            } else if nv.path.is_ident("message") {
                if let Expr::Lit(expr_lit) = &nv.value {
                    if let Lit::Str(s) = &expr_lit.lit {
                        message = Some(s.value());
                    }
                }
            }
        }
    }
    if refer.is_none() && path.is_none() {
        return Err(Error::new(
            args.span(),
            "refer and path cannot be empty at the same time",
        ));
    }
    Ok(FuncCheck {
        refer,
        path,
        message,
        span: Some(args.span()),
    })
}

// GroupCheck
pub fn parse_group_check(args: &Punctuated<Meta, Token![,]>) -> Result<Vec<LitStr>> {
    let mut groups = vec![];
    for meta in args {
        match meta {
            Meta::Path(id) => {
                if let Some(ident) = id.get_ident() {
                    groups.push(LitStr::new(&ident.to_string(), ident.span()));
                }
            }
            Meta::NameValue(nv) => {
                if let Expr::Lit(expr_lit) = &nv.value {
                    if let Lit::Str(s) = &expr_lit.lit {
                        groups.push(s.clone());
                    }
                }
            }
            Meta::List(list) => {
                let exprs =
                    list.parse_args_with(Punctuated::<Expr, Token![,]>::parse_terminated)?;
                for expr in exprs {
                    // 这里 LitStr 只能继续解析
                    if let Expr::Lit(expr_lit) = expr {
                        if let Lit::Str(s) = expr_lit.lit {
                            groups.push(s.clone());
                        }
                    }
                }
            }
        }
    }
    Ok(groups)
}
