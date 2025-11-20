use jiuziai_macro_core::regexes_static;

#[regexes_static]
#[allow(dead_code)]
pub mod patterns {
    // 一定要是字面量 const &str 初始化
    pub const EMAIL: &str = r"^[\w.+-]+@[\w.-]+\.[a-zA-Z]{2,}$";
    pub const DIGITS: &str = r"^\d+$";
    pub const HEX_VALUE: &str = r"^[0-9a-fA-F]+$";
}

#[cfg(test)]
mod tests {
    use crate::patterns::Patterns;
    #[test]
    fn test_regex_static() {
        // 使用枚举获取静态 Regex（第一次访问会编译并打印/执行正则 new）
        let ok = Patterns::HEX_VALUE.regex().is_match("foo@example.com");
        println!("email ok: {}", ok);

        // 也可以通过 name 查找
        if let Some(p) = Patterns::from_name("DIGITS") {
            println!("123 matches DIGITS: {}", p.regex().is_match("123"));
        }

        // 列出所有已生成的名字
        println!("names: {:?}", Patterns::names());
    }
}
