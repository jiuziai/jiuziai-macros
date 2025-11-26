/// Main validation trait
pub trait Validate {
    /// Validate all fields
    fn check(&self) -> Result<bool, String>;

    /// Validate fields with specific group
    fn check_with_group(&self, group: &dyn std::any::Any) -> Result<bool, String>;
}
