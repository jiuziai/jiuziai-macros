use jiuziai_macro_core::Error;
use jiuziai_macro_libs::e;

#[derive(Error)]
#[allow(unused)]
struct RunErrDef {
    #[e(code = "E001", desc = "错误测试: {}")]
    error_test_1: (),
    #[e(code = "E002", desc = "错误测试: {} {}")]
    error_test_2: (),
    #[e(code = "E003", desc = "错误测试: {} {} {}")]
    error_test_3: (),
}

#[macro_use]
macro_rules! run_err {
    () => {
        &RUNERR
    };
}
#[cfg(test)]
mod tests {
    use crate::RUNERR;
    use rust_decimal::Decimal;
    use std::str::FromStr;

    #[test]
    fn test_error_pool() {
        let e1 = run_err!().error_test_1.with_arg("ABC-123".to_string());
        let e2 = run_err!()
            .error_test_2
            .with_arg(Decimal::from_str("!11").unwrap_or(Decimal::ZERO)).with_arg("ABC-123".to_string());
        let e3 = run_err!().error_test_3.with_arg(123.19).with_arg("ABC-123".to_string()).with_arg("ABC-321".to_string());

        eprintln!("e1: {}", serde_json::to_string(&e1).unwrap_or("{}".to_string()));
        eprintln!("e2: {}", serde_json::to_string(&e2).unwrap_or("{}".to_string()));
        eprintln!("e3: {}", serde_json::to_string(&e3).unwrap_or("{}".to_string()));
    }
}
