use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct E {
    pub code: &'static str,
    pub desc: &'static str,
}
impl E {
    pub fn new(code: &'static str, desc: &'static str) -> Self {
        Self { code, desc }
    }
    pub fn get_code(&self) -> String {
        self.code.to_string()
    }
    pub fn get_desc(&self) -> String {
        self.code.to_string()
    }
}

#[macro_export]
macro_rules! e {
    ($code:expr, $desc:expr) => {
        E::new($code, $desc)
    };
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn macro_rules_test() {
        let e1 = e!("E0001", "错误消息");
        println!("{}-{}", e1.get_code(), e1.get_desc())
    }
}
