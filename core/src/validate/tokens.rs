use proc_macro2::TokenStream;
use quote::quote;
use syn::{Data, DeriveInput, Fields, Type, TypePath};

pub fn derive_validate_gen(input: DeriveInput) -> TokenStream {
    match input.data {
        Data::Struct(data_struct) => {
            match data_struct.fields {
                Fields::Named(fields_named) => {
                    // 具名字段：struct S { a: i32, b: String }
                    for field in fields_named.named.iter() {
                        // 字段名字（Some for named fields）
                        let field_ident = field.ident.as_ref().unwrap();
                        // 字段类型 (syn::Type)
                        let field_ty = &field.ty;
                        // 字段的 attrs (Vec<Attribute>)
                        let attrs = &field.attrs;

                        // 你可以按需处理 attrs，例如查找 #[validate(...)]
                        for attr in attrs {
                            if attr.path().is_ident("validate") {
                                // 解析 attr，或把 attr.tokens 转为字符串供调试
                                // let meta = attr.parse_meta().unwrap();
                            }
                        }

                        // 如果你想判断是否为路径类型 (e.g. String, Option<T>, Vec<T>)
                        if let Type::Path(TypePath { path, .. }) = field_ty {
                            // path.segments 是路径段集合
                            if let Some(last_segment) = path.segments.last() {
                                let ty_ident = &last_segment.ident;
                                // ty_ident.to_string() -> "String" / "Option" / ...
                                // 处理泛型参数可以查看 last_segment.arguments
                            }
                        }

                        // TODO: 根据字段信息生成相应的 TokenStream (验证逻辑等)
                    }
                }

                Fields::Unnamed(fields_unnamed) => {
                    // 元组结构体：struct T(i32, String);
                    for (idx, field) in fields_unnamed.unnamed.iter().enumerate() {
                        let index = idx;
                        let field_ty = &field.ty;
                        // ...同上处理 type/attrs
                    }
                }

                Fields::Unit => {
                    // 单元结构体，无字段
                }
            }
        }

        Data::Enum(data_enum) => {
            // 枚举：遍历每个 variant
            for variant in data_enum.variants.iter() {
                let variant_ident = &variant.ident;
                match &variant.fields {
                    Fields::Named(named) => {
                        for field in named.named.iter() {
                            // 读取 variant 的具名字段
                            let field_ident = field.ident.as_ref().unwrap();
                            let field_ty = &field.ty;
                            // 处理同上
                        }
                    }
                    Fields::Unnamed(unnamed) => {
                        for (i, field) in unnamed.unnamed.iter().enumerate() {
                            // 读取 variant 的元组字段
                            let _idx = i;
                            let field_ty = &field.ty;
                        }
                    }
                    Fields::Unit => {
                        // 变体无字段
                    }
                }
            }
        }

        Data::Union(_data_union) => {
            // 一般较少用到，按需处理
        }
    }
    let token_stream = quote! {};
    TokenStream::from(token_stream)
}
