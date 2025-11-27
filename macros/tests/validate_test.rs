#[cfg(test)]
mod tests {
    use jiuziai_libs::validate::types::Validate;
    use jiuziai_macros::{RegexPool, Validator};

    // 测试用的自定义验证函数
    fn custom_validate_email(email: &str) -> bool {
        email.contains('@')
    }

    // 测试用的Regex引用
    #[allow(dead_code)]
    #[derive(RegexPool)]
    struct RegexPoolTestDef {
        #[regex(r"^[a-zA-Z0-9._%+-]+@[a-zA-Z0-9.-]+\.[a-zA-Z]{2,}$")]
        email: (),
        #[regex(r"^1[3-9]\d{9}$")]
        phone: (),
    }

    // 测试用的枚举分组
    #[derive(PartialEq, Debug)]
    enum UserGroup {
        Admin,
        User,
    }

    // 嵌套的验证结构
    #[derive(Validator)]
    struct Profile {
        #[check(required(message = "邮箱必填"))]
        email: Option<String>,

        #[check(range(min = 18, max = 100, message = "年龄必须在18-100之间"))]
        age: u32,
    }

    // 主测试结构
    #[derive(Validator)]
    struct TestUser {
        // any 模式测试
        #[check(required, not_empty, size(min = 3, max = 20), message = "必填或者")]
        username: Option<String>,

        // all 模式测试
        #[check(
            required(message = "密码必填"),
            not_blank(message = "密码不能全是空白字符"),
            no_space(message = "密码不能包含空格"),
            size(min = 6, message = "密码至少6位")
        )]
        password: Option<String>,

        // 正则测试
        #[check(
            required(message = "邮箱必填"),
            regex(refer(REGEX_POOL_TEST.email), message = "邮箱格式错误")
        )]
        email: Option<String>,

        // 自定义函数测试
        #[check(
            required(message = "手机号必填"),
            func(handler(custom_validate_email), message = "手机号格式错误")
        )]
        phone: Option<String>,

        // 数值范围测试
        #[check(
            range(min = 1, max = 150, message = "年龄必须在1-150之间"),
            within(values(18, 25, 30, 40, 50), message = "年龄不在允许范围内")
        )]
        age: u32,

        // 排除值测试
        #[check(out_of(values("admin", "root", "system"), message = "不能使用保留用户名"))]
        role: String,

        // 深度验证测试
        #[check(required(message = "个人资料必填"), deep)]
        profile: Option<Profile>,

        // 分组验证测试
        #[check(required(message = "管理员字段必填"), group(UserGroup::Admin))]
        admin_field: Option<String>,

        // 另一个分组
        #[check(size(min = 1, message = "用户列表不能为空"), group(UserGroup::User))]
        user_list: Option<Vec<String>>,

        // 多分组测试
        #[check(
            required(message = "多分组字段必填"),
            group(UserGroup::Admin, UserGroup::User)
        )]
        multi_group_field: Option<String>,
    }

    #[test]
    fn test_all_validations() {
        let valid_user = TestUser {
            username: Some("john_doe".to_string()),
            password: Some("securepassword".to_string()),
            email: Some("test@example.com".to_string()),
            phone: Some("test@phone.com".to_string()),
            age: 25,
            role: "member".to_string(),
            profile: Some(Profile {
                email: Some("profile@example.com".to_string()),
                age: 25,
            }),
            admin_field: Some("admin_data".to_string()),
            user_list: Some(vec!["user1".to_string()]),
            multi_group_field: Some("multi_data".to_string()),
        };

        // 测试普通验证
        assert!(valid_user.check().is_ok());

        // 测试分组验证
        assert!(valid_user.check_with_group(&UserGroup::Admin).is_ok());
        assert!(valid_user.check_with_group(&UserGroup::User).is_ok());

        // 测试无效数据
        let invalid_user = TestUser {
            username: Some("ab".to_string()),         // 太短
            password: Some("   ".to_string()),        // 全是空白
            email: Some("invalid-email".to_string()), // 格式错误
            phone: Some("invalid".to_string()),       // 自定义验证失败
            age: 200,                                 // 超出范围
            role: "admin".to_string(),                // 在排除列表中
            profile: Some(Profile {
                email: None, // 必填但为空
                age: 15,     // 年龄太小
            }),
            admin_field: None,       // 必填但为空
            user_list: Some(vec![]), // 空列表
            multi_group_field: None, // 必填但为空
        };

        // 验证应该失败
        assert!(invalid_user.check().is_err());
    }

    #[test]
    fn test_any_mode() {
        let user = TestUser {
            username: Some("valid".to_string()), // 这个通过就够
            password: None,                      // 这个失败没关系
            email: None,
            phone: None,
            age: 25,
            role: "member".to_string(),
            profile: None,
            admin_field: None,
            user_list: None,
            multi_group_field: None,
        };

        // any 模式下，只要 username 验证通过就返回成功
        assert!(user.check().is_ok());
    }

    #[test]
    fn test_collection_validation() {
        #[derive(Validator)]
        struct CollectionTest {
            #[check(
                required(message = "列表必填"),
                deep(regex(pattern = r"^[0-9]{1,3}$", message = "不满足正则"))
            )]
            items: Option<Vec<Option<String>>>,
        }

        // let valid = CollectionTest {
        //     items: Some(vec![Some("item1".to_string()), Some("item2".to_string())]),
        // };
        // assert!(valid.check().is_ok());

        let invalid = CollectionTest {
            items: Some(vec![Some("0a".to_string()), None, None]), // 包含空元素
        };
        match invalid.check() {
            Ok(r) => eprintln!("OK: {}", r),
            Err(r) => eprintln!("Error: {}", r),
        }
        // assert!(invalid.check().is_ok());
    }
}
