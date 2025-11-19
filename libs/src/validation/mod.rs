//! 通用验证库
//!
//! 本模块提供一组可复用的验证函数，以及 `Validate` trait 的定义。
//!
//! 目标：为派生宏（`validator` crate）或手写实现提供运行时校验工具，
//! 覆盖 README 中描述的大多数常见校验：长度、数值范围、集合大小、正则、空/空格检查、以及调用自定义函数。
//!
//! 注意：派生宏的自动生成逻辑在 `validator` crate 中实现，这里仅包含运行时逻辑与 trait 定义。

use regex::Regex;

/// 验证 trait
///
/// 实现该 trait 的类型可以执行两种校验：
/// - `check`：对整个结构体不分组地执行所有字段的校验。
/// - `check_group`：只对标注了某个分组的字段进行校验（分组类型由实现者指定）。
///
/// 注意：README 中建议分组可以使用外部字符串（`&str`）或者枚举。分组的用途是标识字段所属的组，
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
    /// 返回 `Ok(true)` 表示通过；`Err(String)` 表示验证失败，并携带开发者提供的错误消息。
    fn check(&self) -> Result<bool, String>;

    /// 只验证标注了指定 `group` 的字段
    ///
    /// 返回规则同上。
    fn check_group(&self, group: Self::Group) -> Result<bool, String>;
}

/// 常用的验证帮助函数集合
pub mod helpers {
    use super::*;

    /// 校验字符串长度（以字符数计）
    ///
    /// - `min`/`max` 可选；若均为 `None` 则视为通过。
    /// - 返回 `Ok(true)` 表示通过；`Err(msg)` 表示失败。
    pub fn validate_len_str(value: &str, min: Option<usize>, max: Option<usize>, message: &str) -> Result<bool, String> {
        let len = value.chars().count();
        if let Some(minv) = min {
            if len < minv {
                return Err(message.to_string());
            }
        }
        if let Some(maxv) = max {
            if len > maxv {
                return Err(message.to_string());
            }
        }
        Ok(true)
    }

    /// 校验数值范围（用 i128 支持大范围整数）
    pub fn validate_range_i128<T: Into<i128>>(value: T, min: Option<i128>, max: Option<i128>, message: &str) -> Result<bool, String> {
        let v = value.into();
        if let Some(minv) = min {
            if v < minv {
                return Err(message.to_string());
            }
        }
        if let Some(maxv) = max {
            if v > maxv {
                return Err(message.to_string());
            }
        }
        Ok(true)
    }

    /// 校验集合大小（用 slice.len()）
    pub fn validate_size_len(len: usize, min: Option<usize>, max: Option<usize>, message: &str) -> Result<bool, String> {
        if let Some(minv) = min {
            if len < minv {
                return Err(message.to_string());
            }
        }
        if let Some(maxv) = max {
            if len > maxv {
                return Err(message.to_string());
            }
        }
        Ok(true)
    }

    /// 不能包含空白字符（空格、制表等）
    pub fn validate_no_space(value: &str, message: &str) -> Result<bool, String> {
        if value.chars().any(|c| c.is_whitespace()) {
            Err(message.to_string())
        } else {
            Ok(true)
        }
    }

    /// 不能为空（字符串或集合）
    pub fn validate_not_empty_str(value: &str, message: &str) -> Result<bool, String> {
        if value.is_empty() {
            Err(message.to_string())
        } else {
            Ok(true)
        }
    }

    /// 去掉两端空白后不能为空
    pub fn validate_not_blank(value: &str, message: &str) -> Result<bool, String> {
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

    /// 正则校验，`pattern` 为有效正则表达式。
    /// 如果传入的 `pattern` 是常量字符串，也能正常工作。
    pub fn validate_regex(value: &str, pattern: &str, message: &str) -> Result<bool, String> {
        // 编译正则并匹配
        let re = Regex::new(pattern).map_err(|e| format!("regex compile error: {}", e))?;
        if re.is_match(value) {
            Ok(true)
        } else {
            Err(message.to_string())
        }
    }

    /// 枚举包含性校验（通过枚举值列表判等）
    ///
    /// 这是对已经是枚举类型的字段进行判等的便捷函数：将字段值与允许的枚举值列表逐个比较（使用 `PartialEq`）。
    pub fn validate_enum<T: PartialEq + Clone>(value: &T, allowed: &[T], message: &str) -> Result<bool, String> {
        if allowed.iter().any(|a| a == value) {
            Ok(true)
        } else {
            Err(message.to_string())
        }
    }

    /// 使用 `TryFrom<Prim>` 的方式校验是否属于某个枚举（兼容 `num_enum::FromPrimitive` 的派生实现）
    ///
    /// 场景：字段值不是枚举类型，而是某个原始整型（比如 `u8` 或 `i32`），想判断它是否能转成目标枚举。
    /// `num_enum::FromPrimitive` 派生会为目标枚举实现 `TryFrom<Prim>`，因此可以用此方法进行校验。
    ///
    /// 示例：`validate_enum_try_from::<MyEnum, u8>(val, "必须是枚举值之一")`。
    pub fn validate_enum_try_from<E, P>(primitive: P, message: &str) -> Result<bool, String>
    where
        E: std::convert::TryFrom<P>,
    {
        if E::try_from(primitive).is_ok() {
            Ok(true)
        } else {
            Err(message.to_string())
        }
    }
}

#[cfg(test)]
mod tests {
    use super::helpers::*;

    #[test]
    fn test_len() {
        assert!(validate_len_str("abc", Some(1), Some(5), "err").is_ok());
        assert!(validate_len_str("", Some(1), None, "err").is_err());
    }

    #[test]
    fn test_range() {
        assert!(validate_range_i128(10i32, Some(5), Some(20), "err").is_ok());
        assert!(validate_range_i128(1i32, Some(5), None, "err").is_err());
    }

    #[test]
    fn test_regex() {
        assert!(validate_regex("12345", r"^[0-9]+$", "err").is_ok());
        assert!(validate_regex("abc", r"^[0-9]+$", "err").is_err());
    }

    #[test]
    fn test_enum_try_from() {
        #[derive(Debug, PartialEq)]
        enum E {
            A = 1,
            B = 2,
        }

        // 为测试的方便性，我们手动实现 TryFrom<u8> 模拟 num_enum 的行为
        impl std::convert::TryFrom<u8> for E {
            type Error = ();
            fn try_from(v: u8) -> Result<Self, Self::Error> {
                match v {
                    1 => Ok(E::A),
                    2 => Ok(E::B),
                    _ => Err(()),
                }
            }
        }

        assert!(validate_enum_try_from::<E, u8>(1u8, "err").is_ok());
        assert!(validate_enum_try_from::<E, u8>(3u8, "err").is_err());
    }
}
