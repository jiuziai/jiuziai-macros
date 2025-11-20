/// 类型名检查工具（按最后一段 ident 判断）
///
/// 说明：这些函数只是基于类型名称字符串做快速判断，适合在 proc-macro 解析时在
///      取到 `path.segments.last().ident` 后调用。对于复杂类型（引用、数组、切片、
///      泛型包装）请先剥离/递归检查 inner type，再使用这些函数。

const RANGE_NAMES: &[&str] = &[
    // 有符号整数
    "i8", "i16", "i32", "i64", "i128", "isize",
    // 无符号整数
    "u8", "u16", "u32", "u64", "u128", "usize",
    // 浮点
    "f32", "f64",
    // 字符
    "char",
    // 定点/高精度十进制类型（如 rust_decimal::Decimal）
    "Decimal",
];

const LEN_NAMES: &[&str] = &[
    // 字符串相关
    "str", "String", "Cow",
    // 常见集合容器
    "Vec", "VecDeque", "LinkedList", "BinaryHeap",
    "HashMap", "HashSet", "BTreeMap", "BTreeSet",
    // 其它常用容器/集合/映射
    "SmallVec", "IndexMap", "IndexSet",
];

const STRING_LIKE: &[&str] = &[
    "str", "String", "Cow",
];

/// 可校验范围的类型（数值 / char / Decimal 等）
///
/// 语义：对于这些类型你通常可以做大小/范围比较（注意 f32/f64 是 PartialOrd，要处理 NaN）。
pub fn range_able(ident: &str) -> bool {
    RANGE_NAMES.contains(&ident)
}

/// 可校验长度的类型（等价于 len/is_empty 的目标）
///
/// 语义：对这些类型可以安全地调用 `.len()` 或等价方法来判断长度/是否为空。
pub fn len_able(ident: &str) -> bool {
    LEN_NAMES.contains(&ident)
}

/// 判断是否适合做“no space”校验（即检查是否包含空白字符）
///
/// 语义：通常用于字符串字段或单字符字段（char）
/// 示例：String/&str/Cow<'_, str>/char
pub fn no_space(ident: &str) -> bool {
    // 包含字符串类与 char
    STRING_LIKE.contains(&ident) || ident == "char"
}

/// 判断是否适合做“not empty”（非空）校验
///
/// 语义：与 len_able 等价（容器/字符串可判空）。如果你希望 Option 也算，可以在解析泛型时特殊处理。
pub fn not_empty_able(ident: &str) -> bool {
    len_able(ident)
}

/// 判断是否适合做“not blank”（去掉空白后是否为空）校验
///
/// 语义：通常只对字符串类有意义（&str / String / Cow<'_, str>）
/// 注意：char 通常不是“blank”概念（除非你明确把 whitespace 的单字符当作 blank）。
pub fn not_blank_able(ident: &str) -> bool {
    STRING_LIKE.contains(&ident)
}

/// 判断是否适合用正则校验（regex）
///
/// 语义：正则通常应用于字符串数据（str/String/Cow），因此只包含字符串类。
pub fn regex_able(ident: &str) -> bool {
    STRING_LIKE.contains(&ident)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_range() {
        assert!(range_able("i32"));
        assert!(range_able("u64"));
        assert!(range_able("f64"));
        assert!(range_able("char"));
        assert!(range_able("Decimal"));
        assert!(!range_able("String"));
        assert!(!range_able("Vec"));
    }

    #[test]
    fn test_len_and_empty() {
        assert!(len_able("String"));
        assert!(len_able("str"));
        assert!(len_able("Vec"));
        assert!(!len_able("i32"));
        assert!(not_empty_able("Vec"));
        assert!(!not_empty_able("i32"));
    }

    #[test]
    fn test_string_checks() {
        assert!(no_space("String"));
        assert!(no_space("str"));
        assert!(no_space("char"));
        assert!(!no_space("i64"));

        assert!(not_blank_able("String"));
        assert!(!not_blank_able("Vec"));

        assert!(regex_able("str"));
        assert!(regex_able("String"));
        assert!(!regex_able("i32"));
    }
}