use crate::validate::parse::field_meta::{FieldValidation, parse_field_attributes};
use syn::{Data, DeriveInput, Fields};
use syn::spanned::Spanned;

/// 解析结构体的所有字段属性
pub fn parse_struct_attributes(input: &DeriveInput) -> Result<Vec<FieldValidation>, syn::Error> {
    let mut fields_validation = Vec::new();

    match &input.data {
        Data::Struct(data_struct) => match &data_struct.fields {
            Fields::Named(fields_named) => {
                for field in &fields_named.named {
                    let field_validation = parse_field_attributes(field)?;
                    fields_validation.push(field_validation);
                }
            }
            Fields::Unnamed(_) => {
                return Err(syn::Error::new(
                    input.span(),
                    "Validate derive macro only supports structs with named fields",
                ));
            }
            Fields::Unit => {
                return Err(syn::Error::new(
                    input.span(),
                    "Validate derive macro does not support unit structs",
                ));
            }
        },
        Data::Enum(_) => {
            return Err(syn::Error::new(
                input.span(),
                "Validate derive macro only supports structs, not enums",
            ));
        }
        Data::Union(_) => {
            return Err(syn::Error::new(
                input.span(),
                "Validate derive macro only supports structs, not unions",
            ));
        }
    }

    Ok(fields_validation)
}
