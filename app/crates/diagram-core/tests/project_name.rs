use diagram_core::{InvalidProjectName, ProjectName};

#[test]
fn new_should_reject_empty_project_names() {
    assert_eq!(ProjectName::new(String::new()), Err(InvalidProjectName));
}

#[test]
fn new_should_reject_unicode_whitespace_only_names() {
    assert_eq!(
        ProjectName::new(String::from(" \t\n\u{2003}\u{3000}")),
        Err(InvalidProjectName)
    );
}

#[test]
fn new_should_preserve_mixed_language_spelling() -> Result<(), InvalidProjectName> {
    let name = ProjectName::new(String::from("  Customer نموذج  "))?;

    assert_eq!(name.as_str(), "  Customer نموذج  ");
    Ok(())
}
