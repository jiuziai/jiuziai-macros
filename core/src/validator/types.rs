use proc_macro2::{Ident, Span};
use syn::{Expr, Path, Type};

#[derive(Debug, Clone)]
pub struct FieldInfo {
    pub name: Ident,
    pub ty: Type,
    pub inner_ty: Option<Type>,

    pub required: Option<BoolCheck>,
    pub not_empty: Option<BoolCheck>,
    pub not_blank: Option<BoolCheck>,
    pub no_space: Option<BoolCheck>,
    pub size: Option<MinMaxCheck>,
    pub range: Option<MinMaxCheck>,
    pub within: Option<ValuesCheck>,
    pub out_of: Option<ValuesCheck>,
    pub regex: Option<RegexCheck>,
    pub func: Option<FuncCheck>,

    pub deep: Option<DeepCheck>,
    pub message: Option<String>,
    pub group: Option<Vec<Path>>,
    pub span: Span,
}

impl FieldInfo {
    pub fn new(name: Ident, ty: Type, span: Span) -> Self {
        Self {
            name,
            ty,
            inner_ty: None,
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
            group: None,
            span,
        }
    }
}
#[derive(Debug, Clone)]
pub struct DeepCheck {
    pub ty: Type,
    pub inner_ty: Option<Type>,

    pub required: Option<BoolCheck>,
    pub not_empty: Option<BoolCheck>,
    pub not_blank: Option<BoolCheck>,
    pub no_space: Option<BoolCheck>,
    pub size: Option<MinMaxCheck>,
    pub range: Option<MinMaxCheck>,
    pub within: Option<ValuesCheck>,
    pub out_of: Option<ValuesCheck>,
    pub regex: Option<RegexCheck>,
    pub func: Option<FuncCheck>,

    pub message: Option<String>,
    pub span: Span,
}
#[derive(Debug, Clone)]
pub struct BoolCheck {
    pub message: Option<String>,
    pub span: Span,
}
#[derive(Debug, Clone)]
pub struct MinMaxCheck {
    pub min: Option<Expr>,
    pub max: Option<Expr>,
    pub message: Option<String>,
    pub span: Span,
}
#[derive(Debug, Clone)]
pub struct ValuesCheck {
    pub values: Vec<Expr>,
    pub message: Option<String>,
    pub span: Span,
}
#[derive(Debug, Clone)]
pub struct RegexCheck {
    pub refer: Option<Expr>,
    pub pattern: Option<String>,
    pub message: Option<String>,
    pub span: Span,
}
#[derive(Debug, Clone)]
pub struct FuncCheck {
    pub handler: Expr,
    pub message: Option<String>,
    pub span: Span,
}
