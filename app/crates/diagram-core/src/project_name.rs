use std::fmt;

use thiserror::Error;

/// An owned project name containing at least one non-whitespace character.
///
/// Spelling, surrounding whitespace and Unicode are preserved. Presentation
/// normalization belongs to an explicit user action rather than construction.
///
/// # Examples
///
/// ```
/// use diagram_core::ProjectName;
///
/// let name = ProjectName::new(String::from("Customer model"))?;
/// assert_eq!(name.as_str(), "Customer model");
/// # Ok::<(), diagram_core::InvalidProjectName>(())
/// ```
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ProjectName(String);

impl ProjectName {
    /// Takes ownership of a name without changing its spelling.
    ///
    /// # Errors
    ///
    /// Returns [`InvalidProjectName`] when the name is empty or entirely
    /// Unicode whitespace.
    pub fn new(value: String) -> Result<Self, InvalidProjectName> {
        if value.trim().is_empty() {
            return Err(InvalidProjectName);
        }
        Ok(Self(value))
    }

    /// Borrows the name without allocating or cloning it.
    pub fn as_str(&self) -> &str {
        &self.0
    }
}

impl fmt::Display for ProjectName {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter.write_str(self.as_str())
    }
}

/// A project name was empty or contained only whitespace.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Error)]
#[error("a project name must contain a non-whitespace character")]
pub struct InvalidProjectName;
