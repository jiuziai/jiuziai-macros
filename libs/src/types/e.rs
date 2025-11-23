use serde::ser::SerializeStruct;
use serde::{Deserialize, Serialize, Serializer};

/// 错误信息结构体
///
/// # ⚠ 警告
///
/// ### **禁止业务代码直接构造 `E` 实例**！
/// > 为提高性能，本结构体内含 `&'static str` 字段，可能造成大量重复分配静态数据，或引发严重的内存安全问题，如过非要业务代码构造 `E` 实例，禁止将变量赋值给 `&'static str` 相关字段。
/// # 正确用法：
/// ```rust
/// use jiuziai_macro_core::Error;
/// use jiuziai_macro_libs::types::e::E;
///
/// #[derive(Error)] // Error派生宏会生成静态错误池 MY_SERVICE_ERROR
/// #[allow(unused)]
/// struct MyServiceErrorDef{
///     #[e(code="E0001", desc="用户 {} 未找到")]
///     user_not_found: (),
///     #[e(code="E0002", desc="订单 {} 已取消")]
///     order_cancelled: (),
/// }
///
/// fn example_usage()-> E {
///   MY_SERVICE_ERROR.user_not_found.with_arg("Alice")
/// }
/// ```
///
#[derive(Debug, Clone, Deserialize)]
pub struct E {
    pub code: &'static str,
    pub desc: &'static str,
    #[serde(skip)]
    pub template: &'static [&'static str],
    #[serde(skip)]
    pub args: Vec<String>,
}

impl E {
    pub fn new(code: &'static str, desc: &'static str) -> Self {
        let template: &'static [&'static str] = if desc.contains("{}") {
            let owned_vec: Vec<&'static str> = desc.split("{}").collect();
            Box::leak(owned_vec.into_boxed_slice())
        } else {
            &[]
        };
        let desc_ref_static: &'static str = Box::leak(
            desc.replace(r"\{", "{")
                .replace(r"\}", "}")
                .into_boxed_str(),
        );
        Self {
            code,
            desc: desc_ref_static,
            template,
            args: Vec::new(),
        }
    }
    pub fn get_code(&self) -> String {
        self.code.to_string()
    }
    pub fn get_desc(&self) -> String {
        if self.template.is_empty() || self.args.is_empty() {
            return self.desc.to_string();
        }
        let mut desc = String::new();
        let l = self.template.len();
        let n = self.args.len();
        for i in 0..l {
            desc.push_str(self.template[i]);
            if i < l - 1 {
                if i < n {
                    desc.push_str(&self.args[i]);
                } else {
                    desc.push_str("{}");
                }
            }
        }
        desc
    }

    pub fn get(&self) -> Self {
        self.clone()
    }
    pub fn with_arg(&self, arg: impl ToString) -> Self {
        Self {
            code: self.code,
            desc: self.desc,
            template: self.template.clone(),
            args: {
                let mut new_args = self.args.clone();
                new_args.push(arg.to_string());
                new_args
            },
        }
    }
}
impl Serialize for E {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        let mut map = serializer.serialize_struct("E", 2)?;
        map.serialize_field("code", &self.get_code())?;
        map.serialize_field("desc", &self.get_desc())?;
        map.end()
    }
}

#[macro_export]
macro_rules! e {
    ($code:expr, $desc:expr) => {
        E::new($code, $desc)
    };
}
#[cfg(test)]
mod tests {
    use super::*;
    use once_cell::sync::Lazy;
    struct A {
        e: E,
    }
    // 静态延迟初始化 A_ERROR
    static A_ERROR: Lazy<A> = Lazy::new(|| A {
        e: E::new("100", "错误消息: {},{}"),
    });

    #[test]
    fn macro_rules_test() {
        let e1 = A_ERROR.e.clone().with_arg(0).with_arg(1).with_arg(2);
        // 输出：E0001-错误消息
        println!("{}-{}", e1.get_code(), e1.get_desc());
    }
}
