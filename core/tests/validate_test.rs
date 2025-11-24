#[allow(dead_code)]
#[cfg(test)]
mod tests {
    use jiuziai_macro_core::Validator;
    use jiuziai_macro_libs::validate::types::ValidateTrait;
    fn validate_test() -> bool {
        true
    }

    enum MyEnum {
        A,
        B,
        C,
    }

    #[derive(Validator)]
    struct SimpleUser {
        #[check(
            required(message = "名字必填"),
            not_empty(message = "名字不能为空"),
            not_blank(message = "名字不能为空白字符"),
            no_space(message = "名字不能包含空格"),
            size(min = 3, max = 16, message = "名字长度不符合要求"),
            range(min = 1, max = 100, message = "名字长度不符合要求"),
            within(values(1, 2, 3), message = "名字不在允许范围内"),
            out_of(values(3, 2, 1), message = "名字在禁止范围内"),
            regex(
                refer(REGEX_POOL.email),
                pattern = r"asdfkasdlkjaklsdjflkasjdfklasjdflkajsdlkfj",
                message = "名字格式错误"
            ),
            func(
                handler(validate_test),
                message="名字格式错误",
            ),
            deep,
            message = "名字格式错误",
            group(MyEnum::A, MyEnum::B),
        )]
        name: u64,
    }


    #[test]
    fn test_simple() {
        let _user = SimpleUser {
            name: Some("test".to_string()),
        };
    }
}
