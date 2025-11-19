use jiuziai_validator::Validate as _derive;

// We call the runtime trait directly from jiuziai_macro_libs
#[test]
fn option_require_fails() {
    #[derive(_derive)]
    struct OptReq {
        #[validate(check(require(message = "id required")))]
        id: Option<i32>,
    }

    let a = OptReq { id: None };
    let res = jiuziai_macro_libs::validation::Validate::check(&a);
    assert!(res.is_err());
    assert_eq!(res.unwrap_err(), "id required");
}

#[test]
fn vec_nested_validate_failure() {
    #[derive(_derive)]
    struct Inner {
        #[validate(check(not_blank(message = "name blank")))]
        name: String,
    }

    #[derive(_derive)]
    struct Outer {
        inners: Vec<Inner>,
    }

    let o = Outer { inners: vec![Inner { name: "".to_string() }] };
    let res = jiuziai_macro_libs::validation::Validate::check(&o);
    
    assert!(res.is_err());
    assert_eq!(res.unwrap_err(), "name blank");
}
