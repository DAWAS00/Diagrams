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

#[test]
fn new_should_accept_a_single_non_whitespace_character() -> Result<(), InvalidProjectName> {
    let name = ProjectName::new(String::from("x"))?;

    assert_eq!(name.as_str(), "x");
    Ok(())
}

#[test]
fn new_should_accept_names_with_inner_whitespace_and_line_breaks() -> Result<(), InvalidProjectName>
{
    let name = ProjectName::new(String::from("Order\nmodel\t2"))?;

    assert_eq!(name.as_str(), "Order\nmodel\t2");
    Ok(())
}

#[test]
fn display_should_match_the_stored_spelling() -> Result<(), InvalidProjectName> {
    let name = ProjectName::new(String::from(" Customer نموذج "))?;

    assert_eq!(name.to_string(), " Customer نموذج ");
    assert_eq!(format!("[{name}]"), "[ Customer نموذج ]");
    Ok(())
}

#[test]
fn equality_should_compare_exact_spelling() -> Result<(), InvalidProjectName> {
    let name = ProjectName::new(String::from("Model"))?;

    assert_eq!(name.clone(), name);
    assert_ne!(name, ProjectName::new(String::from("model"))?);
    assert_ne!(name, ProjectName::new(String::from("Model "))?);
    Ok(())
}

#[test]
fn invalid_project_name_should_explain_the_rule() {
    assert_eq!(
        InvalidProjectName.to_string(),
        "a project name must contain a non-whitespace character"
    );
}

#[test]
fn invalid_project_name_should_be_a_standard_error() {
    let error: Box<dyn std::error::Error> = Box::new(InvalidProjectName);

    assert!(error.source().is_none());
}
