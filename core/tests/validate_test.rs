#[cfg(test)]
mod tests {
    use jiuziai_macro_core::Validator;
    use jiuziai_macro_libs::validate::ValidateTrait;
    #[derive(Validator)]
    struct SimpleUser {
        #[check(required(message = "名字必填"))]
        name: Option<String>,
    }

    #[test]
    fn test_simple() {
        let user = SimpleUser {
            name: Some("test".to_string()),
        };
        let result = user.check();
        match result {
            Ok(ok) => eprintln!("result: {:?}", ok.to_string()),
            Err(err) => println!("error: {:?}", err),
        }
    }

    #[test]
    fn test_required_fail() {
        let user = SimpleUser { name: None };
        let result = user.check();
        assert!(result.is_err());
    }
}
