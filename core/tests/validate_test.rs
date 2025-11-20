#[cfg(test)]
mod tests {
    use super::*;
    use serde::{Deserialize, Serialize};
    use regex::Regex;
    use jiuziai_macro_core::Validate;

    // 测试用的枚举
    #[derive(Debug, Clone, PartialEq)]
    enum TestEnum {
        Unit1,
        Unit2,
        Unit3,
    }

    // 测试用的模式结构体
    struct Patterns;
    impl Patterns {
        const HEX_VALUE: &'static str = "^[0-9a-fA-F]+$";

        pub fn hex_value_regex() -> Regex {
            Regex::new(Self::HEX_VALUE).unwrap()
        }
    }

    // 测试函数
    fn test111(_s: &str) -> bool {
        true
    }

    #[derive(Debug, Clone, PartialEq, Deserialize, Serialize, Validate)]
    struct A {
        #[size(min = 5, max = 10, message = "长度必须在5-10之间")]
        #[func(func = test111, message = "自定义函数验证失败")]
        #[within(values = ["x", "y"], message = "必须在x或y中")]
        #[required(message = "字段不能为空")]
        #[not_blank]
        #[regex(pattern = Patterns::hex_value_regex(), message = "必须是十六进制格式")]
        #[no_space]
        #[message = "自定义错误消息"]
        pub f1: Option<String>,

        #[within(values = [1, 2, 3], message = "必须在1,2,3中")]
        #[exclude(values = [1, 3, 4])]
        #[not_empty(message = "不能为空")]
        #[group(groups = [TestEnum::Unit1, TestEnum::Unit2])] // 枚举值
        pub f2: Vec<u64>,
    }

    #[test]
    fn test1() {
        let a = A {
            f1: Some("abc123".to_string()),
            f2: vec![2, 5],
        };

        // 使用示例
        // let result = a.check_group(TestEnum::Unit1);
        // let result = a.check_group(TestEnum::Unit2);
    }
}