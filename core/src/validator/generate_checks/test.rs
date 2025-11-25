Macro expansion:
impl jiuziai_macro_libs::validate::types::Validate for TestUser {
    // any 模式测试 
    pub fn check(&self) -> Result<bool, String> {
        if self.username.is_some() { return Ok(true); }
        if if let Some(inner) = &self.username { !self.username.is_empty() } else { true } { return Ok(true); }
        if if let Some(inner) = &self.username { self.username.chars().count() >= 3 && self.username.chars().count() <= 20 } else { true } { return Ok(true); }
        return Err("用户名验证失败");
        if self.password.is_none() { return Err("密码必填"); }
        if if let Some(inner) = &self.password { !(!self.password.trim().is_empty()) } else { true } { return Err("密码不能全是空白字符"); }
        if if let Some(inner) = &self.password { !(!self.password.chars().any(|c| c.is_whitespace())) } else { true } { return Err("密码不能包含空格"); }
        if if let Some(inner) = &self.password { !(self.password.chars().count() >= 6 && true) } else { true } { return Err("密码至少6位"); }
        return Ok(true);
        if self.email.is_none() { return Err("邮箱必填"); }
        ::core::compile_error! {"`regex` check `refer/pattern` must have a valid value" }
        return Ok(true);
        if self.phone.is_none() { return Err("手机号必填"); }
        if if let Some(inner) = &self.phone { !(custom_validate_email(&self.phone)) } else { true } { return Err("手机号格式错误"); }
        return Ok(true);
        if !(self.age >= 1 && self.age <= 150) { return Err("年龄必须在1-150之间"); }
        if !([18, 25, 30, 40, 50].contains(self.age)) { return Err("年龄不在允许范围内"); }
        return Ok(true);
        if !(!["admin", "root", "system"].contains(self.role)) { return Err("不能使用保留用户名"); }
        return Ok(true);
        if self.profile.is_none() { return Err("个人资料必填"); }
        if let Some(inner_value) = &self.profile { inner_value.check()?; }
        return Ok(true);
        if self.admin_field.is_none() { return Err("管理员字段必填"); }
        return Ok(true);
        if if let Some(inner) = &self.user_list { !(self.user_list.len() >= 1 && true) } else { true } { return Err("用户列表不能为空"); }
        return Ok(true);
        if self.multi_group_field.is_none() { return Err("多分组字段必填"); }
        return Ok(true);
        Ok(true)
    }
    pub fn check_with_group(&self, group: impl PartialEq) -> Result<bool, String> {
        match group {
            UserGroup::Admin => {
                if self.admin_field.is_none() { return Err("管理员字段必填"); }
                return Ok(true);
                if self.multi_group_field.is_none() { return Err("多分组字段必填"); }
                return Ok(true);
                Ok(true)
            }
            UserGroup::User => {
                if if let Some(inner) = &self.user_list { !(self.user_list.len() >= 1 && true) } else { true } { return Err("用户列表不能为空"); }
                return Ok(true);
                if self.multi_group_field.is_none() { return Err("多分组字段必填"); }
                return Ok(true);
                Ok(true)
            }
            _ => self.check()
        }
        Ok(true)
    }
}