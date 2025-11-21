/// Main validation trait
pub trait ValidateTrait {
    /// Validate all fields
    fn check(&self) -> Result<bool, String>;

    /// Validate fields with specific group
    fn check_with_group(&self, group: &str) -> Result<bool, String>;
}
