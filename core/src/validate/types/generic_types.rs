use syn::Type;
use super::basic_types::BasicValidationType;

/// 支持泛型的有效类型 - 泛型参数必须是基础类型
#[derive(Debug, Clone)]
pub enum GenericValidationType {
    Basic(BasicValidationType),
    Option(Box<GenericValidationType>),
    Vec(Box<GenericValidationType>),
    HashSet(Box<GenericValidationType>),
    HashMap(Box<GenericValidationType>, Box<GenericValidationType>),
}

impl GenericValidationType {
    pub fn from_type(ty: &Type) -> Self {
        Self::from_type_inner(ty, 0)
    }

    fn from_type_inner(ty: &Type, depth: u32) -> Self {
        if depth > 5 {
            return GenericValidationType::Basic(BasicValidationType::Unsupported);
        }

        if let Type::Path(type_path) = ty {
            let path = &type_path.path;
            if let Some(segment) = path.segments.last() {
                match segment.ident.to_string().as_str() {
                    "Option" => {
                        if let syn::PathArguments::AngleBracketed(args) = &segment.arguments {
                            if let Some(syn::GenericArgument::Type(inner_ty)) = args.args.first() {
                                let inner = Self::from_type_inner(inner_ty, depth + 1);
                                if inner.is_valid() {
                                    return GenericValidationType::Option(Box::new(inner));
                                }
                            }
                        }
                        GenericValidationType::Basic(BasicValidationType::Unsupported)
                    }
                    "Vec" => {
                        if let syn::PathArguments::AngleBracketed(args) = &segment.arguments {
                            if let Some(syn::GenericArgument::Type(inner_ty)) = args.args.first() {
                                let inner = Self::from_type_inner(inner_ty, depth + 1);
                                // Vec 的泛型参数必须是基础类型
                                if let GenericValidationType::Basic(basic) = &inner {
                                    if basic.is_collection_compatible() {
                                        return GenericValidationType::Vec(Box::new(inner));
                                    }
                                }
                            }
                        }
                        GenericValidationType::Basic(BasicValidationType::Unsupported)
                    }
                    "HashSet" => {
                        if let syn::PathArguments::AngleBracketed(args) = &segment.arguments {
                            if let Some(syn::GenericArgument::Type(inner_ty)) = args.args.first() {
                                let inner = Self::from_type_inner(inner_ty, depth + 1);
                                // HashSet 的泛型参数必须是基础类型
                                if let GenericValidationType::Basic(basic) = &inner {
                                    if basic.is_collection_compatible() {
                                        return GenericValidationType::HashSet(Box::new(inner));
                                    }
                                }
                            }
                        }
                        GenericValidationType::Basic(BasicValidationType::Unsupported)
                    }
                    "HashMap" => {
                        if let syn::PathArguments::AngleBracketed(args) = &segment.arguments {
                            let mut args_iter = args.args.iter();
                            if let (
                                Some(syn::GenericArgument::Type(key_ty)),
                                Some(syn::GenericArgument::Type(value_ty))
                            ) = (args_iter.next(), args_iter.next()) {
                                let key_inner = Self::from_type_inner(key_ty, depth + 1);
                                let value_inner = Self::from_type_inner(value_ty, depth + 1);

                                // HashMap 的键和值必须是基础类型
                                if let (GenericValidationType::Basic(key_basic), GenericValidationType::Basic(value_basic)) = (&key_inner, &value_inner) {
                                    if key_basic.is_collection_compatible() && value_basic.is_collection_compatible() {
                                        return GenericValidationType::HashMap(Box::new(key_inner), Box::new(value_inner));
                                    }
                                }
                            }
                        }
                        GenericValidationType::Basic(BasicValidationType::Unsupported)
                    }
                    _ => {
                        // 基础类型
                        GenericValidationType::Basic(BasicValidationType::from_type(ty))
                    }
                }
            } else {
                GenericValidationType::Basic(BasicValidationType::Unsupported)
            }
        } else {
            GenericValidationType::Basic(BasicValidationType::Unsupported)
        }
    }

    pub fn is_valid(&self) -> bool {
        match self {
            GenericValidationType::Basic(basic) => !matches!(basic, BasicValidationType::Unsupported),
            GenericValidationType::Option(inner) => inner.is_valid(),
            GenericValidationType::Vec(inner) => inner.is_valid(),
            GenericValidationType::HashSet(inner) => inner.is_valid(),
            GenericValidationType::HashMap(key, value) => key.is_valid() && value.is_valid(),
        }
    }

    pub fn get_base_type(&self) -> &BasicValidationType {
        match self {
            GenericValidationType::Basic(basic) => basic,
            GenericValidationType::Option(inner) => inner.get_base_type(),
            GenericValidationType::Vec(inner) => inner.get_base_type(),
            GenericValidationType::HashSet(inner) => inner.get_base_type(),
            GenericValidationType::HashMap(key, _) => key.get_base_type(),
        }
    }

    pub fn is_string(&self) -> bool {
        self.get_base_type().is_string()
    }

    pub fn is_collection(&self) -> bool {
        matches!(self, GenericValidationType::Vec(_) | GenericValidationType::HashSet(_) | GenericValidationType::HashMap(_, _))
    }

    pub fn supports_range(&self) -> bool {
        self.get_base_type().supports_range()
    }

    pub fn supports_size(&self) -> bool {
        self.is_string() || self.is_collection()
    }

    pub fn is_option(&self) -> bool {
        matches!(self, GenericValidationType::Option(_))
    }

    pub fn is_custom_struct(&self) -> bool {
        matches!(self.get_base_type(), BasicValidationType::CustomStruct)
    }
}