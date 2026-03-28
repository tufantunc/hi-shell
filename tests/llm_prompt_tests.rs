use hi_shell::llm::{get_system_info, get_system_prompt};

#[test]
fn test_system_prompt_contains_compatibility_rules() {
    let prompt = get_system_prompt(None);
    assert!(prompt.contains("COMPATIBILITY RULES"));
    assert!(prompt.contains("macOS"));
    assert!(prompt.contains("Windows"));
}

#[test]
fn test_system_prompt_contains_schema() {
    let prompt = get_system_prompt(None);
    assert!(prompt.contains("command"));
    assert!(prompt.contains("explanation"));
    assert!(prompt.contains("dangerous"));
}

#[test]
fn test_system_prompt_with_repair_context() {
    let prompt = get_system_prompt(Some("No such file or directory"));
    assert!(prompt.contains("REPAIR RULES"));
    assert!(prompt.contains("No such file or directory"));
    assert!(prompt.contains("DO NOT suggest the same failing command"));
}

#[test]
fn test_system_prompt_without_repair_context() {
    let prompt = get_system_prompt(None);
    assert!(!prompt.contains("REPAIR RULES"));
}

#[test]
fn test_system_prompt_includes_os_info() {
    let prompt = get_system_prompt(None);
    assert!(prompt.contains("Operating System"));
    assert!(prompt.contains("Shell"));
}

#[test]
fn test_system_prompt_includes_cwd_files() {
    let prompt = get_system_prompt(None);
    assert!(prompt.contains("Current Working Directory"));
    assert!(prompt.contains("Files in CWD"));
}

#[test]
fn test_system_info_format() {
    let info = get_system_info();
    assert!(info.starts_with("Operating System:"));
    assert!(info.contains("Shell:"));
    assert!(info.contains("Current Working Directory:"));
    assert!(info.contains("Files in CWD:"));
}

#[test]
fn test_system_info_files_limit() {
    let info = get_system_info();
    let files_section = info.split("Files in CWD: ").nth(1).unwrap_or("");
    let file_count = if files_section.is_empty() {
        0
    } else {
        files_section.split(", ").count()
    };
    assert!(file_count <= 50);
}
