#[cfg(test)]
mod tests {
    use jiuziai_macro_core::Validate;
    use serde::{Deserialize, Serialize};

    #[derive(Debug, Clone, PartialEq, Deserialize, Serialize, Validate)]
    struct A {
        #[validate(
            len(min = 1, max = 2),
            message = "10",
            func(ident = "test111", message = "test111")
        )]
        pub f1: String,
        #[validate(
            range(min = 1, max = 2, message = ""),
            message = "10",
            func(ident = "test111", message = "test111"),
            deep
        )]
        pub f2: String,
    }
    #[test]
    fn test1() {
        let _a = A {
            f1: "asdfasdf".to_string(),
            f2: "asdf".to_string(),
        };
    }
}
