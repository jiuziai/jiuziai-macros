use crate::validator::boundary::*;
use crate::validator::types::*;
use quote::ToTokens;
use std::collections::HashSet;
use syn::{
    punctuated::Punctuated, spanned::Spanned, Data, DataStruct, DeriveInput, Error, Expr, ExprPath, Field, Fields, Lit,
    Meta, MetaNameValue, Result, Token,
};

/// 解析整个 struct 输入，返回所有带 check 的字段（FieldInfo 列表）
pub fn parse_struct(input: &DeriveInput) -> Result<Vec<MetaInfo>> {
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
pub fn parse_field(field: &Field) -> Result<MetaInfo> {
    let depth:u8 = 0;
    let name = field
        .ident
        .clone()
        .ok_or_else(|| Error::new(field.span(), "Field must have ident"))?;
    let ty = field.ty.clone();
    let inner_ty = is_option_type(&ty).then(|| strip_option(&ty));
    let mut info = MetaInfo::new(name, ty, inner_ty, field.span());
    for attr in &field.attrs {
        if attr.path().is_ident("check") {
            let metas: Punctuated<Meta, Token![,]> =
                attr.parse_args_with(Punctuated::<Meta, Token![,]>::parse_terminated)?;
            for meta in &metas {
                parse_check_meta(meta, &mut info, depth)?;
            }
        }
    }
    check_field_info_boundary(&mut info, depth)?;
    Ok(info)
}

/// 递归解析 #[check(...)] 每个 Meta
pub fn parse_check_meta(meta: &Meta, info: &mut MetaInfo, depth: u8) -> Result<()> {
    match meta {
        Meta::Path(path) => {
            if let Some(ident) = path.get_ident() {
                match ident.to_string().as_str() {
                    "deep" => {
                        if depth > 0 {
                            return Err(Error::new(
                                path.span(),
                                "Does not allow deeper depth verification",
                            ));
                        }
                        if info.deep.is_some() {
                            return Err(Error::new(path.span(), "`deep` attribute already exists"));
                        }
                        info.deep = Some(Box::from(MetaInfo::new(
                            info.name.clone(),
                            info.ty.clone(),
                            info.option_ty.clone(),
                            path.span(),
                        )))
                    }
                    "traits" => {
                        if depth == 0 {
                            return Err(Error::new(
                                path.span(),
                                "The `traits` attribute is not allowed at the top level",
                            ));
                        }
                        exists_check(&info.traits, path.span(), "traits")?;
                        info.traits = Some(BoolCheck::new(None, path.span()));
                    }
                    "required" => {
                        exists_check(&info.required, path.span(), "required")?;
                        info.required = Some(BoolCheck::new(None, path.span()));
                    }
                    "not_empty" => {
                        exists_check(&info.not_empty, path.span(), "not_empty")?;
                        info.not_empty = Some(BoolCheck::new(None, path.span()));
                    }
                    "not_blank" => {
                        exists_check(&info.not_blank, path.span(), "not_blank")?;
                        info.not_blank = Some(BoolCheck::new(None, path.span()));
                    }
                    "no_space" => {
                        exists_check(&info.no_space, path.span(), "no_space")?;
                        info.no_space = Some(BoolCheck::new(None, path.span()));
                    }
                    _ => {
                        return Err(Error::new(
                            path.span(),
                            format!("Unknown unary check: {}", ident),
                        ));
                    }
                }
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
                        if info.message.is_some() {
                            return Err(Error::new(
                                expr_lit.span(),
                                "`message` attribute already exists",
                            ));
                        }
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
                "required" => parse_required_check(info, &args)?,
                "not_empty" => parse_not_empty_check(info, &args)?,
                "not_blank" => parse_not_blank_check(info, &args)?,
                "no_space" => parse_no_space_check(info, &args)?,
                "size" | "len" => parse_size_check(info, &args)?,
                "range" => parse_range_check(info, &args)?,
                "within" => parse_within_check(info, &args)?,
                "out_of" => parse_out_of_check(info, &args)?,
                "regex" => parse_regex_check(info, &args)?,
                "func" => parse_func_check(info, &args)?,
                "group" => {
                    if depth > 0 {
                        return Err(Error::new(
                            group_name.span(),
                            "Deep validation does not allow the use of `group`",
                        ));
                    }
                    parse_group_check(info, &args)
                }?,
                "traits" => {
                    if depth == 0 {
                        return Err(Error::new(
                            group_name.span(),
                            "The `traits` attribute is not allowed at the top level",
                        ));
                    }
                    parse_traits_check(info, &args)?
                }
                "deep" => {
                    if depth > 1 {
                        return Err(Error::new(
                            group_name.span(),
                            "Does not allow deeper depth verification",
                        ));
                    }

                    // 创建 deep 信息
                    let mut deep_info = MetaInfo::new(
                        info.name.clone(),
                        info.ty.clone(),
                        info.option_ty.clone(),
                        group_name.span(),
                    );

                    // if is_collection(&deep_info.ty) {
                    //     let elem_ty = get_collection_element_type(
                    //         &info.name,
                    //         &info.span,
                    //         &strip_option(&info.ty),
                    //     )?;
                    //     deep_info.ty = elem_ty.clone();
                    //     if is_option_type(&deep_info.ty) {
                    //         deep_info.option_ty = Some(strip_option(&deep_info.ty));
                    //     } else {
                    //         deep_info.option_ty = None
                    //     }
                    // }

                    // 遍历 args 中的每个 meta，分别解析
                    for meta in &args {
                        parse_check_meta(meta, &mut deep_info, depth + 1)?;
                    }

                    info.deep = Some(Box::new(deep_info));
                }
                _ => {
                    return Err(Error::new(
                        group_name.span(),
                        format!("Unknown check group: {}", group_name),
                    ));
                }
            }
        }
    }
    Ok(())
}

// RequiredCheck

pub fn parse_required_check(info: &mut MetaInfo, args: &Punctuated<Meta, Token![,]>) -> Result<()> {
    exists_check(&info.required, args.span(), "required")?;
    info.required = Some(parse_bool(args)?);
    Ok(())
}
// RequiredCheck

pub fn parse_not_empty_check(
    info: &mut MetaInfo,
    args: &Punctuated<Meta, Token![,]>,
) -> Result<()> {
    exists_check(&info.not_empty, args.span(), "not_empty")?;
    info.not_empty = Some(parse_bool(args)?);
    Ok(())
}

// RequiredCheck

pub fn parse_not_blank_check(
    info: &mut MetaInfo,
    args: &Punctuated<Meta, Token![,]>,
) -> Result<()> {
    exists_check(&info.not_blank, args.span(), "not_blank")?;
    info.not_blank = Some(parse_bool(args)?);
    Ok(())
}

// RequiredCheck

pub fn parse_no_space_check(info: &mut MetaInfo, args: &Punctuated<Meta, Token![,]>) -> Result<()> {
    exists_check(&info.no_space, args.span(), "no_space")?;
    info.no_space = Some(parse_bool(args)?);
    Ok(())
}

// SizeCheck
pub fn parse_size_check(info: &mut MetaInfo, args: &Punctuated<Meta, Token![,]>) -> Result<()> {
    exists_check(&info.size, args.span(), "size")?;
    info.size = Some(parse_min_max(args)?);
    Ok(())
}
// RangeCheck
pub fn parse_range_check(info: &mut MetaInfo, args: &Punctuated<Meta, Token![,]>) -> Result<()> {
    exists_check(&info.range, args.span(), "range")?;
    info.range = Some(parse_min_max(args)?);
    Ok(())
}
// WithinCheck
pub fn parse_within_check(info: &mut MetaInfo, args: &Punctuated<Meta, Token![,]>) -> Result<()> {
    exists_check(&info.within, args.span(), "within")?;
    info.within = Some(parse_values(args)?);
    Ok(())
}
// WithOutCheck
pub fn parse_out_of_check(info: &mut MetaInfo, args: &Punctuated<Meta, Token![,]>) -> Result<()> {
    exists_check(&info.out_of, args.span(), "out_of")?;
    info.out_of = Some(parse_values(args)?);
    Ok(())
}

// RegexCheck
pub fn parse_regex_check(info: &mut MetaInfo, args: &Punctuated<Meta, Token![,]>) -> Result<()> {
    exists_check(&info.regex, args.span(), "regex")?;

    let mut refer: Option<Expr> = None; // 改为 Expr 类型
    let mut pattern = None;
    let mut message = None;

    for meta in args {
        match meta {
            Meta::List(list) if list.path.is_ident("refer") => {
                if refer.is_some() {
                    return Err(Error::new(
                        list.span(),
                        "`refer` can only be specified once",
                    ));
                }
                // 直接解析为 Expr，支持路径和字段访问
                let expr: Expr = list.parse_args()?;
                refer = Some(expr);
            }
            Meta::NameValue(nv) if nv.path.is_ident("pattern") => {
                exists_param(&pattern, nv.span(), "pattern")?;
                if let Expr::Lit(expr_lit) = &nv.value {
                    if let Lit::Str(s) = &expr_lit.lit {
                        pattern = Some(s.value());
                    } else {
                        return Err(Error::new(
                            expr_lit.lit.span(),
                            "pattern must be a string literal",
                        ));
                    }
                } else {
                    return Err(Error::new(
                        nv.value.span(),
                        "pattern must be a string literal",
                    ));
                }
            }
            Meta::NameValue(nv) if nv.path.is_ident("message") => parse_message(&mut message, nv)?,
            _ => {
                return Err(Error::new(
                    meta.span(),
                    "Only `refer(...)`, `pattern = \"...\"` and `message = \"...\"` are allowed in regex check",
                ));
            }
        }
    }

    if refer.is_none() && pattern.is_none() {
        return Err(Error::new(
            args.span(),
            "Either `refer` or `pattern` must be specified",
        ));
    }

    info.regex = Some(RegexCheck {
        refer,
        pattern,
        message,
        span: args.span(),
    });

    Ok(())
}

// FuncCheck
pub fn parse_func_check(info: &mut MetaInfo, args: &Punctuated<Meta, Token![,]>) -> Result<()> {
    exists_check(&info.func, args.span(), "func")?;

    let mut handler: Option<Expr> = None;
    let mut message = None;

    for meta in args {
        match meta {
            Meta::List(list) if list.path.is_ident("handler") => {
                if handler.is_some() {
                    return Err(Error::new(
                        list.span(),
                        "`handler` can only be specified once",
                    ));
                }
                let expr: Expr = list.parse_args()?;
                handler = Some(expr);
            }
            Meta::NameValue(nv) if nv.path.is_ident("message") => parse_message(&mut message, nv)?,
            _ => {
                return Err(Error::new(
                    meta.span(),
                    "Only `handler(...)` and `message=\"...\"` are allowed in `func`",
                ));
            }
        }
    }

    let handler = handler.ok_or_else(|| Error::new(args.span(), "`handler` is required"))?;

    info.func = Some(FuncCheck {
        handler,
        message,
        span: args.span(),
    });

    Ok(())
}

// TraitCheck
pub fn parse_traits_check(info: &mut MetaInfo, args: &Punctuated<Meta, Token![,]>) -> Result<()> {
    exists_check(&info.traits, args.span(), "traits")?;

    if !args.is_empty() {
        return Err(Error::new(
            args.span(),
            "`traits` cannot contain any parameters",
        ));
    }

    info.traits = Some(BoolCheck {
        message: None,
        span: args.span(),
    });

    Ok(())
}

// GroupCheck
pub fn parse_group_check(info: &mut MetaInfo, args: &Punctuated<Meta, Token![,]>) -> Result<()> {
    if info.group.is_some() {
        return Err(Error::new(args.span(), "`group` attribute already exists"));
    }

    let mut groups: Vec<Expr> = Vec::new();

    for meta in args {
        match meta {
            Meta::Path(path) => {
                // 将 Path 转为 ExprPath
                let expr = Expr::Path(ExprPath {
                    attrs: Vec::new(),
                    qself: None,
                    path: path.clone(),
                });
                groups.push(expr);
            }
            _ => {
                return Err(Error::new(
                    meta.span(),
                    "`group` only allows path expressions like MyEnum::A",
                ));
            }
        }
    }

    if groups.is_empty() {
        return Err(Error::new(args.span(), "`group` cannot be empty"));
    }

    // 检查重复
    let mut seen = HashSet::new();
    for expr in &groups {
        let key = expr.to_token_stream().to_string();
        if !seen.insert(key.clone()) {
            return Err(Error::new(
                expr.span(),
                format!("duplicate group \"{}\" found", key),
            ));
        }
    }

    info.group = Some(groups);
    Ok(())
}
// 解析 Values 类型
pub fn parse_bool(args: &Punctuated<Meta, Token![,]>) -> Result<BoolCheck> {
    let mut message = None;
    // 检查是否有空括号的情况：required()
    if args.is_empty() {
        // 这是 required 不带参数的情况，允许
        return Ok(BoolCheck {
            message: None,
            span: args.span(),
        });
    }
    for meta in args {
        match meta {
            Meta::NameValue(nv) if nv.path.is_ident("message") => parse_message(&mut message, nv)?,
            _ => {
                return Err(Error::new(
                    meta.span(),
                    "Only `message = \"...\"` or no arguments are allowed for bool check",
                ));
            }
        }
    }
    Ok(BoolCheck {
        message,
        span: args.span(),
    })
}

// 解析 Values 类型
pub fn parse_values(args: &Punctuated<Meta, Token![,]>) -> Result<ValuesCheck> {
    let mut values: Vec<Expr> = vec![];
    let mut message = None;
    let mut has_values = false;

    for meta in args {
        match meta {
            Meta::List(list) if list.path.is_ident("values") => {
                if has_values {
                    return Err(Error::new(
                        list.span(),
                        "`values` checker parameters already exists",
                    ));
                }
                has_values = true;

                let expr_list =
                    list.parse_args_with(Punctuated::<Expr, Token![,]>::parse_terminated)?;
                for expr in expr_list {
                    values.push(expr);
                }
            }
            Meta::NameValue(nv) if nv.path.is_ident("message") => parse_message(&mut message, nv)?,
            _ => {
                return Err(Error::new(
                    meta.span(),
                    "Only `values(...)` and `message=\"...\"` are allowed in values check",
                ));
            }
        }
    }

    if !has_values {
        return Err(Error::new(args.span(), "`values(...)` is required"));
    }

    if values.is_empty() {
        return Err(Error::new(args.span(), "values() must not be empty"));
    }

    Ok(ValuesCheck {
        values,
        message,
        span: args.span(),
    })
}

// 解析 MinMax 类型
pub fn parse_min_max(args: &Punctuated<Meta, Token![,]>) -> Result<MinMaxCheck> {
    let mut min: Option<Expr> = None;
    let mut max: Option<Expr> = None;
    let mut message = None;

    for meta in args {
        if let Meta::NameValue(nv) = meta {
            if nv.path.is_ident("min") {
                exists_param(&min, nv.span(), "min")?;
                min = Some(nv.value.clone());
            } else if nv.path.is_ident("max") {
                exists_param(&max, nv.span(), "max")?;
                max = Some(nv.value.clone());
            } else if nv.path.is_ident("message") {
                parse_message(&mut message, nv)?
            } else {
                return Err(Error::new(
                    nv.path.span(),
                    format!(
                        "Unknown parameter `{}` in size check",
                        nv.path.get_ident().unwrap()
                    ),
                ));
            }
        } else {
            return Err(Error::new(
                meta.span(),
                "Only `min`, `max` and `message` are allowed in size check",
            ));
        }
    }

    if min.is_none() && max.is_none() {
        return Err(Error::new(
            args.span(),
            "min and max cannot be empty at the same time",
        ));
    }

    // 检查 min < max（如果两者都存在）
    if let (Some(min_expr), Some(max_expr)) = (&min, &max) {
        // 这里可以尝试比较字面量，如果是复杂表达式就跳过检查
        if let (Expr::Lit(min_lit), Expr::Lit(max_lit)) = (min_expr, max_expr) {
            if let (Lit::Int(min_int), Lit::Int(max_int)) = (&min_lit.lit, &max_lit.lit) {
                let min_val = min_int.base10_parse::<i64>()?;
                let max_val = max_int.base10_parse::<i64>()?;
                if min_val >= max_val {
                    return Err(Error::new(args.span(), "min must be less than max"));
                }
            }
        }
        // 对于非字面量表达式，我们无法在编译时比较，跳过检查
    }

    Ok(MinMaxCheck {
        min,
        max,
        message,
        span: args.span(),
    })
}

// 解析 Check 消息
pub fn parse_message(message: &mut Option<String>, nv: &MetaNameValue) -> Result<()> {
    exists_param(message, nv.span(), "message")?;
    if let Expr::Lit(expr_lit) = &nv.value {
        if let Lit::Str(s) = &expr_lit.lit {
            *message = Some(s.value());
            Ok(())
        } else {
            Err(Error::new(
                expr_lit.lit.span(),
                "message must be a string literal",
            ))
        }
    } else {
        Err(Error::new(
            nv.value.span(),
            "message must be a string literal",
        ))
    }
}

// 判断 Check 类型是否已存在
pub fn exists_check<T>(
    option: &Option<T>,
    span: proc_macro2::Span,
    check_name: &str,
) -> Result<()> {
    if option.is_some() {
        return Err(Error::new(
            span,
            format!("`{}` validation checker already exists", check_name),
        ));
    }
    Ok(())
}

// 判断 Check 内部参数是否已存在
pub fn exists_param<T>(
    option: &Option<T>,
    span: proc_macro2::Span,
    param_name: &str,
) -> Result<()> {
    if option.is_some() {
        return Err(Error::new(
            span,
            format!("`{}` checker parameters already exists", param_name),
        ));
    }
    Ok(())
}
