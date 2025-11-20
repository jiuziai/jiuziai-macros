use syn::Type;

/// 基础有效类型 - 不允许泛型参数
#[derive(Debug, Clone, PartialEq)]
pub enum BasicValidationType {
    String,
    Integer,
    Float,
    Boolean,
    Decimal,
    DateTime,
    CustomStruct,
    Enum,
    Unsupported,
}

impl BasicValidationType {
    pub fn from_type(ty: &Type) -> Self {
        if let Type::Path(type_path) = ty {
            let path = &type_path.path;
            if let Some(segment) = path.segments.last() {
                match segment.ident.to_string().as_str() {
                    "String" => BasicValidationType::String,
                    "i8" | "i16" | "i32" | "i64" | "i128" | "isize" |
                    "u8" | "u16" | "u32" | "u64" | "u128" | "usize" => BasicValidationType::Integer,
                    "f32" | "f64" => BasicValidationType::Float,
                    "bool" => BasicValidationType::Boolean,
                    "Decimal" => BasicValidationType::Decimal,
                    "DateTime" | "NaiveDateTime" | "NaiveDate" => BasicValidationType::DateTime,
                    _ => {
                        if Self::is_custom_type(path) {
                            BasicValidationType::CustomStruct
                        } else if Self::is_enum_type(path) {
                            BasicValidationType::Enum
                        } else {
                            BasicValidationType::Unsupported
                        }
                    }
                }
            } else {
                BasicValidationType::Unsupported
            }
        } else {
            BasicValidationType::Unsupported
        }
    }

    fn is_custom_type(path: &syn::Path) -> bool {
        let first_segment = path.segments.first().map(|s| s.ident.to_string());
        match first_segment.as_deref() {
            Some("std") | Some("core") | Some("alloc") | Some("chrono") | Some("rust_decimal") => false,
            _ => {
                let last_segment = path.segments.last().map(|s| s.ident.to_string());
                !matches!(
                    last_segment.as_deref(),
                    Some("String") | Some("i8") | Some("i16") | Some("i32") | Some("i64") |
                    Some("i128") | Some("isize") | Some("u8") | Some("u16") | Some("u32") |
                    Some("u64") | Some("u128") | Some("usize") | Some("f32") | Some("f64") |
                    Some("bool") | Some("Decimal") | Some("DateTime") | Some("NaiveDateTime") |
                    Some("NaiveDate") | Some("Vec") | Some("Option") | Some("HashSet") |
                    Some("HashMap")
                )
            }
        }
    }

    fn is_enum_type(path: &syn::Path) -> bool {
        // 这里可以添加枚举类型的检测逻辑
        // 暂时返回 false，在实际使用中需要根据具体情况实现
        false
    }

    pub fn supports_range(&self) -> bool {
        matches!(self, BasicValidationType::Integer | BasicValidationType::Float | BasicValidationType::Decimal | BasicValidationType::DateTime)
    }

    pub fn is_string(&self) -> bool {
        matches!(self, BasicValidationType::String)
    }

    pub fn is_collection_compatible(&self) -> bool {
        !matches!(self, BasicValidationType::Unsupported)
    }

    pub fn supports_within_exclude(&self) -> bool {
        !matches!(self, BasicValidationType::Unsupported)
    }
}