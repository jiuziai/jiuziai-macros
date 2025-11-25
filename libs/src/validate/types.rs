/// Main validation trait
pub trait Validate {
    /// Validate all fields
    fn check(&self) -> Result<bool, String>;

    /// Validate fields with specific group
    fn check_with_group(&self, group: impl PartialEq) -> Result<bool, String>;
}
