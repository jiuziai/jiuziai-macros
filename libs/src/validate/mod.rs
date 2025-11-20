//! 通用验证库
//!
//! 本模块提供一组可复用的验证函数，以及 `Validate` trait 的定义。
//!
//! 目标：为派生宏（`core` crate）或手写实现提供运行时校验工具，
//! 覆盖 README 中描述的大多数常见校验：长度、数值范围、集合大小、正则、空/空格检查、以及调用自定义函数。
//!
//! 注意：派生宏的自动生成逻辑在 `core` crate 中实现，这里仅包含运行时逻辑与 trait 定义。

use regex::Regex;

/// 验证 trait
///
/// 实现该 trait 的类型可以执行两种校验：
/// - `check`：对整个结构体不分组地执行所有字段的校验。
/// - `check_group`：只对标注了某个分组的字段进行校验（分组类型由实现者指定）。
///
/// 在实现 `check_group` 时会使用 `PartialEq` 来判断字段上声明的分组是否等于传入的 `group` 值。
/// 如果你选择用枚举作为分组，通常会要求该枚举实现 `Debug, Clone, Serialize, Deserialize, PartialEq`（由派生宏或使用处决定）。
pub trait Validate {
    /// 分组类型
    ///
    /// 分组类型用于 `check_group` 的比较，必须实现 `PartialEq`。
    /// 具体的额外 trait 约束（如 `Serialize` / `Deserialize` / `Debug` / `Clone`）由派生宏或调用方决定是否要求。
    type Group: PartialEq;

    /// 对结构体的所有字段执行验证（忽略分组）
    ///
    /// 返回 `Ok(true)` 表示通过；`err(String)` 表示验证失败，并携带开发者提供的错误消息。
    fn check(&self) -> Result<bool, String>;

    /// 只验证标注了指定 `group` 的字段
    ///
    /// 返回规则同上。
    fn check_group(&self, group: Self::Group) -> Result<bool, String>;
}

/// 常用的验证帮助函数集合
pub mod helpers {
    use super::*;

    /// 校验字符串、数组长度（兼容中文）
    pub fn validate_len(
        value: &String,
        min: Option<usize>,
        max: Option<usize>,
        message: &str,
    ) -> Result<bool, String> {
        let len = value.chars().count();
        if let Some(min_v) = min {
            if len < min_v {
                return Err(message.to_string());
            }
        }
        if let Some(max_v) = max {
            if len > max_v {
                return Err(message.to_string());
            }
        }
        Ok(true)
    }

    /// 校验数值范围（用 i128 支持大范围整数）
    pub fn validate_range<T: Into<i128>>(
        value: T,
        min: Option<i128>,
        max: Option<i128>,
        message: &str,
    ) -> Result<bool, String> {
        let v = value.into();
        if let Some(min_v) = min {
            if v < min_v {
                return Err(message.to_string());
            }
        }
        if let Some(max_v) = max {
            if v > max_v {
                return Err(message.to_string());
            }
        }
        Ok(true)
    }

    /// 不能包含空白字符（空格、制表等）
    pub fn validate_no_space(value: &String, message: &str) -> Result<bool, String> {
        if value.chars().any(|c| c.is_whitespace()) {
            Err(message.to_string())
        } else {
            Ok(true)
        }
    }

    /// 不能为空（字符串或集合）
    pub fn validate_not_empty(value: &String, message: &str) -> Result<bool, String> {
        if value.is_empty() {
            Err(message.to_string())
        } else {
            Ok(true)
        }
    }

    /// 去掉两端空白后不能为空
    pub fn validate_not_blank(value: &String, message: &str) -> Result<bool, String> {
        if value.trim().is_empty() {
            Err(message.to_string())
        } else {
            Ok(true)
        }
    }

    /// 调用自定义函数进行验证，用户需提供 `Fn(&T) -> bool` 函数或闭包
    pub fn validate_func<T, F>(value: &T, func: F, message: &str) -> Result<bool, String>
    where
        F: Fn(&T) -> bool,
    {
        if func(value) {
            Ok(true)
        } else {
            Err(message.to_string())
        }
    }

    /// 正则校验，`pattern` 为有效正则表达式`Regex`类型，不支持`&str`类型实时编译，性能低且无法结合属性宏对正则表达式进行编译期检查
    pub fn validate_regex(value: &str, re: &Regex, message: &str) -> Result<bool, String> {
        if re.is_match(value) {
            Ok(true)
        } else {
            Err(message.to_string())
        }
    }

    /// 枚举包含性校验
    pub fn validate_enum<T: PartialEq>(
        value: &T,
        allowed: &[T],
        message: &str,
    ) -> Result<bool, String> {
        if allowed.iter().any(|a| a == value) {
            Ok(true)
        } else {
            Err(message.to_string())
        }
    }

}
