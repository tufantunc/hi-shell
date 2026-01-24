use hi_shell::llm::{get_system_info, parse_llm_response};

#[test]
fn test_parse_llm_response_json() {
    let input = r#"{"command": "ls -la", "explanation": "list files", "dangerous": false}"#;
    let res = parse_llm_response(input).unwrap();
    assert_eq!(res.command, "ls -la");
    assert!(!res.dangerous);
}

#[test]
fn test_parse_llm_response_markdown() {
    let input = r#"Here is the command:
```json
{
  "command": "rm -rf /",
  "explanation": "danger!",
  "dangerous": true
}
```"#;
    let res = parse_llm_response(input).unwrap();
    assert_eq!(res.command, "rm -rf /");
    assert!(res.dangerous);
}

#[test]
fn test_system_info_contains_essential_fields() {
    let info = get_system_info();
    assert!(info.contains("Operating System"));
    assert!(info.contains("Shell"));
    assert!(info.contains("Current Working Directory"));
}
