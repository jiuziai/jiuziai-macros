use jiuziai_macro_core::RegexPool;

#[allow(dead_code)]
#[derive(RegexPool)]
struct RegexPoolTestDef {
    #[regex(r"^[\w.+-]+@[\w.-]+\.[a-zA-Z]{2,}$")]
    email: (),
    #[regex(r"^\d+$")]
    digits: (),
    #[regex(r"^[0-9a-fA-F]+$")]
    hex_value: (),
}
#[cfg(test)]
mod tests {
    use super::REGEX_POOL_TEST;
    #[test]
    fn test_email_regex() {
        // 测试正确的 email
        let valid = [
            "abc@foo.com",
            "user.name+bar@baz.cn",
            "a-b.c@domain.org",
            "foo@bar.co.uk",
        ];
        let invalid = [
            "plainaddress",
            "abc@foo",
            "@foo.com",
            "foo@bar.",
            "foo@bar.c",
            "a@b",
        ];
        for email in valid {
            assert!(
                REGEX_POOL_TEST.email.is_match(email),
                "Should match: {}",
                email
            );
        }
        for email in invalid {
            assert!(
                !REGEX_POOL_TEST.email.is_match(email),
                "Should NOT match: {}",
                email
            );
        }
    }

    #[test]
    fn test_digits_regex() {
        assert!(
            REGEX_POOL_TEST.digits.is_match("1234567890"),
            "Should match digits"
        );
        assert!(
            REGEX_POOL_TEST.digits.is_match("0"),
            "Should match single digit"
        );
        assert!(
            !REGEX_POOL_TEST.digits.is_match("12ab34"),
            "Should not match letters"
        );
        assert!(!REGEX_POOL_TEST.digits.is_match(""), "Should not match empty");
    }

    #[test]
    fn test_hex_value_regex() {
        let valid = ["0123de", "ABCDEF", "0a1b2c3d4e5f", "abcdef", "123", "a"];
        let invalid = ["GHIJKL", "xyz", "123g", "qwerty", "xyz012", ""];
        for s in valid {
            assert!(REGEX_POOL_TEST.hex_value.is_match(s), "Should match: {}", s);
        }
        for s in invalid {
            assert!(
                !REGEX_POOL_TEST.hex_value.is_match(s),
                "Should NOT match: {}",
                s
            );
        }
    }
}
