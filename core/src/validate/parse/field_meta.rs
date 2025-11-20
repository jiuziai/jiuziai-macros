use syn::{Ident, Type, LitStr, LitInt, Path};
use proc_macro2::TokenStream;
use syn::spanned::Spanned;

/// 解析后的字段验证信息
pub struct FieldValidation {
    pub ident: Ident,
    pub field_type: Type,
    pub func: Option<FuncOptions>,
    pub not_blank: Option<BoolOptions>,
    pub not_empty: Option<BoolOptions>,
    pub no_space: Option<BoolOptions>,
    pub range: Option<RangeOptions>,
    pub regex: Option<TokenStream>,
    pub required: Option<BoolOptions>,
    pub size: Option<SizeOptions>,
    pub within: Option<VecOptions>,
    pub exclude: Option<VecOptions>,
    pub deep: Option<BoolOptions>,
    pub message: Option<String>,
    pub group: Option<Vec<TokenStream>>,
}

pub struct VecOptions {
    pub values: Vec<TokenStream>,
    pub message: Option<String>,
}

pub struct BoolOptions {
    pub message: Option<String>,
}

pub struct FuncOptions {
    pub func: TokenStream,
    pub message: Option<String>,
}

pub struct RangeOptions {
    pub min: Option<i64>,
    pub max: Option<i64>,
    pub message: Option<String>,
}

pub struct SizeOptions {
    pub min: Option<u64>,
    pub max: Option<u64>,
    pub message: Option<String>,
}

/// 解析字段属性
pub fn parse_field_attributes(field: &syn::Field) -> Result<FieldValidation, syn::Error> {
    let ident = field.ident.clone().ok_or_else(|| {
        syn::Error::new(field.span(), "Field must have an identifier")
    })?;

    let field_type = field.ty.clone();

    let mut validation = FieldValidation {
        ident: ident.clone(),
        field_type: field_type.clone(),
        func: None,
        not_blank: None,
        not_empty: None,
        no_space: None,
        range: None,
        regex: None,
        required: None,
        size: None,
        within: None,
        exclude: None,
        deep: None,
        message: None,
        group: None,
    };

    for attr in &field.attrs {
        if attr.path().is_ident("func") {
            validation.func = Some(parse_func_attribute(attr)?);
        } else if attr.path().is_ident("not_blank") {
            validation.not_blank = Some(parse_bool_attribute(attr)?);
        } else if attr.path().is_ident("not_empty") {
            validation.not_empty = Some(parse_bool_attribute(attr)?);
        } else if attr.path().is_ident("no_space") {
            validation.no_space = Some(parse_bool_attribute(attr)?);
        } else if attr.path().is_ident("range") {
            validation.range = Some(parse_range_attribute(attr)?);
        } else if attr.path().is_ident("regex") {
            validation.regex = Some(parse_regex_attribute(attr)?);
        } else if attr.path().is_ident("required") {
            validation.required = Some(parse_bool_attribute(attr)?);
        } else if attr.path().is_ident("size") {
            validation.size = Some(parse_size_attribute(attr)?);
        } else if attr.path().is_ident("within") {
            validation.within = Some(parse_vec_attribute(attr)?);
        } else if attr.path().is_ident("exclude") {
            validation.exclude = Some(parse_vec_attribute(attr)?);
        } else if attr.path().is_ident("deep") {
            validation.deep = Some(parse_bool_attribute(attr)?);
        } else if attr.path().is_ident("message") {
            validation.message = Some(parse_message_attribute(attr)?);
        } else if attr.path().is_ident("group") {
            validation.group = Some(parse_group_attribute(attr)?);
        }
    }

    Ok(validation)
}

fn parse_bool_attribute(attr: &syn::Attribute) -> Result<BoolOptions, syn::Error> {
    let mut message = None;

    let _ = attr.parse_nested_meta(|meta| {
        if meta.path.is_ident("message") {
            let value: LitStr = meta.value()?.parse()?;
            message = Some(value.value());
            Ok(())
        } else {
            Err(meta.error("expected `message`"))
        }
    });

    Ok(BoolOptions { message })
}

fn parse_range_attribute(attr: &syn::Attribute) -> Result<RangeOptions, syn::Error> {
    let mut min = None;
    let mut max = None;
    let mut message = None;

    attr.parse_nested_meta(|meta| {
        if meta.path.is_ident("min") {
            let value: LitInt = meta.value()?.parse()?;
            min = Some(value.base10_parse::<i64>()?);
            Ok(())
        } else if meta.path.is_ident("max") {
            let value: LitInt = meta.value()?.parse()?;
            max = Some(value.base10_parse::<i64>()?);
            Ok(())
        } else if meta.path.is_ident("message") {
            let value: LitStr = meta.value()?.parse()?;
            message = Some(value.value());
            Ok(())
        } else {
            Err(meta.error("expected `min`, `max`, or `message`"))
        }
    })?;

    Ok(RangeOptions { min, max, message })
}

fn parse_size_attribute(attr: &syn::Attribute) -> Result<SizeOptions, syn::Error> {
    let mut min = None;
    let mut max = None;
    let mut message = None;

    attr.parse_nested_meta(|meta| {
        if meta.path.is_ident("min") {
            let value: LitInt = meta.value()?.parse()?;
            min = Some(value.base10_parse::<u64>()?);
            Ok(())
        } else if meta.path.is_ident("max") {
            let value: LitInt = meta.value()?.parse()?;
            max = Some(value.base10_parse::<u64>()?);
            Ok(())
        } else if meta.path.is_ident("message") {
            let value: LitStr = meta.value()?.parse()?;
            message = Some(value.value());
            Ok(())
        } else {
            Err(meta.error("expected `min`, `max`, or `message`"))
        }
    })?;

    Ok(SizeOptions { min, max, message })
}

fn parse_func_attribute(attr: &syn::Attribute) -> Result<FuncOptions, syn::Error> {
    let mut func = None;
    let mut message = None;

    // 使用新的解析方式
    attr.parse_nested_meta(|meta| {
        if meta.path.is_ident("func") {
            let value = meta.value()?;
            let expr: syn::Expr = value.parse()?;
            func = Some(quote::quote!(#expr));
            Ok(())
        } else if meta.path.is_ident("message") {
            let value = meta.value()?;
            let lit: syn::LitStr = value.parse()?;
            message = Some(lit.value());
            Ok(())
        } else {
            Err(meta.error("expected `func` or `message`"))
        }
    })?;

    Ok(FuncOptions {
        func: func.ok_or_else(|| syn::Error::new(attr.span(), "func attribute requires a function expression"))?,
        message,
    })
}
fn parse_regex_attribute(attr: &syn::Attribute) -> Result<proc_macro2::TokenStream, syn::Error> {
    let mut pattern = None;
    let mut message = None;

    attr.parse_nested_meta(|meta| {
        if meta.path.is_ident("pattern") {
            let value = meta.value()?;
            let expr: syn::Expr = value.parse()?;
            pattern = Some(quote::quote!(#expr));
            Ok(())
        } else if meta.path.is_ident("message") {
            let value = meta.value()?;
            let lit: syn::LitStr = value.parse()?;
            message = Some(lit.value());
            Ok(())
        } else {
            Err(meta.error("expected `pattern` or `message`"))
        }
    })?;

    pattern.ok_or_else(|| syn::Error::new(attr.span(), "regex attribute requires a pattern expression"))
}

fn parse_vec_attribute(attr: &syn::Attribute) -> Result<VecOptions, syn::Error> {
    let mut values = Vec::new();
    let mut message = None;

    attr.parse_nested_meta(|meta| {
        if meta.path.is_ident("values") {
            let value = meta.value()?;
            // 解析数组表达式，如 [1, 2, 3]
            let expr: syn::ExprArray = value.parse()?;
            for element in expr.elems {
                values.push(quote::quote!(#element));
            }
            Ok(())
        } else if meta.path.is_ident("message") {
            let value = meta.value()?;
            let lit: syn::LitStr = value.parse()?;
            message = Some(lit.value());
            Ok(())
        } else {
            Err(meta.error("expected `values` or `message`"))
        }
    })?;

    Ok(VecOptions { values, message })
}

fn parse_message_attribute(attr: &syn::Attribute) -> Result<String, syn::Error> {
    let value: LitStr = attr.parse_args()?;
    Ok(value.value())
}

fn parse_group_attribute(attr: &syn::Attribute) -> Result<Vec<syn::Path>, syn::Error> {
    let mut groups = Vec::new();

    attr.parse_nested_meta(|meta| {
        if meta.path.is_ident("groups") {
            let value = meta.value()?;
            // 解析路径数组，如 [TestEnum::Unit1, TestEnum::Unit2]
            let expr: syn::ExprArray = value.parse()?;
            for element in expr.elems {
                if let syn::Expr::Path(path_expr) = element {
                    groups.push(path_expr.path);
                } else {
                    return Err(syn::Error::new(element.span(), "expected path expression"));
                }
            }
            Ok(())
        } else {
            Err(meta.error("expected `groups`"))
        }
    })?;

    Ok(groups)
}