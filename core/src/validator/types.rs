use proc_macro2::Span;

#[derive(Debug, Clone)]
pub struct FieldInfo {
    pub name: syn::Ident,
    pub ty: syn::Type,

    pub required: Option<BoolCheck>,
    pub not_empty: Option<BoolCheck>,
    pub not_blank: Option<BoolCheck>,
    pub no_space: Option<BoolCheck>,
    pub size: Option<SizeCheck>,
    pub range: Option<RangeCheck>,
    pub within: Option<ValuesCheck>,
    pub out_of: Option<ValuesCheck>,
    pub regex: Option<RegexCheck>,
    pub func: Option<FuncCheck>,

    pub deep: Option<EmptyCheck>,
    pub message: Option<String>,
    pub group: Vec<syn::LitStr>,
    pub span: Option<Span>,
}

impl FieldInfo {
    pub fn new(name: syn::Ident, ty: syn::Type) -> Self {
        Self {
            name,
            ty,
            required: None,
            not_empty: None,
            not_blank: None,
            no_space: None,
            size: None,
            range: None,
            within: None,
            out_of: None,
            regex: None,
            func: None,
            deep: None,
            message: None,
            group: vec![],
            span: None,
        }
    }
}
#[derive(Debug, Clone)]
pub struct EmptyCheck {
    pub value: bool,
    pub span: Option<Span>,
}
#[derive(Debug, Clone)]
pub struct BoolCheck {
    pub value: bool,
    pub message: Option<String>,
    pub span: Option<Span>,
}
#[derive(Debug, Clone)]
pub struct SizeCheck {
    pub min: Option<u64>,
    pub max: Option<u64>,
    pub message: Option<String>,
    pub span: Option<Span>,
}
#[derive(Debug, Clone)]
pub struct RangeCheck {
    pub min: Option<syn::Expr>,
    pub max: Option<syn::Expr>,
    pub message: Option<String>,
    pub span: Option<Span>,
}
#[derive(Debug, Clone)]
pub struct ValuesCheck {
    pub values: Vec<syn::Expr>,
    pub message: Option<String>,
    pub span: Option<Span>,
}
#[derive(Debug, Clone)]
pub struct RegexCheck {
    pub refer: Option<syn::Expr>,
    pub pattern: Option<String>,
    pub message: Option<String>,
    pub span: Option<Span>,
}
#[derive(Debug, Clone)]
pub struct FuncCheck {
    pub refer: Option<syn::Expr>,
    pub path: Option<syn::Path>,
    pub message: Option<String>,
    pub span: Option<Span>,
}
