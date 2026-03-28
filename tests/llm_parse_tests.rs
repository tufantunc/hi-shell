use hi_shell::llm::parse_llm_response;

#[test]
fn test_parse_clean_json() {
    let input = r#"{"command": "ls -la", "explanation": "list files", "dangerous": false}"#;
    let res = parse_llm_response(input).unwrap();
    assert_eq!(res.command, "ls -la");
    assert_eq!(res.explanation.as_deref(), Some("list files"));
    assert!(!res.dangerous);
}

#[test]
fn test_parse_json_with_markdown_code_block() {
    let input = r#"Here is the command:
```json
{"command": "rm -rf /", "explanation": "danger!", "dangerous": true}
```"#;
    let res = parse_llm_response(input).unwrap();
    assert_eq!(res.command, "rm -rf /");
    assert!(res.dangerous);
}

#[test]
fn test_parse_json_with_generic_markdown_block() {
    let input = r#"Result:
```
{"command": "pwd", "explanation": "print working directory", "dangerous": false}
```"#;
    let res = parse_llm_response(input).unwrap();
    assert_eq!(res.command, "pwd");
}

#[test]
fn test_parse_json_with_surrounding_text() {
    let input = r#"I will help you with that.
{"command": "git status", "explanation": "check repo status", "dangerous": false}
Hope this helps!"#;
    let res = parse_llm_response(input).unwrap();
    assert_eq!(res.command, "git status");
}

#[test]
fn test_parse_json_missing_explanation() {
    let input = r#"{"command": "ls", "dangerous": false}"#;
    let res = parse_llm_response(input).unwrap();
    assert_eq!(res.command, "ls");
    assert!(res.explanation.is_none());
    assert!(!res.dangerous);
}

#[test]
fn test_parse_json_missing_dangerous() {
    let input = r#"{"command": "echo hello", "explanation": "print hello"}"#;
    let res = parse_llm_response(input).unwrap();
    assert_eq!(res.command, "echo hello");
    assert!(!res.dangerous);
}

#[test]
fn test_parse_json_is_dangerous_field() {
    let input =
        r#"{"command": "rm -rf /tmp/test", "explanation": "remove temp", "is_dangerous": true}"#;
    let res = parse_llm_response(input).unwrap();
    assert!(res.dangerous);
}

#[test]
fn test_parse_json_with_answer_field() {
    let input = r#"{"answer": "find . -name '*.rs'"}"#;
    let res = parse_llm_response(input).unwrap();
    assert_eq!(res.command, "find . -name '*.rs'");
    assert!(res.explanation.is_none());
    assert!(!res.dangerous);
}

#[test]
fn test_parse_nested_json_in_command() {
    let input = r#"{"command": "echo '{\"key\": \"value\"}'", "explanation": "nested json", "dangerous": false}"#;
    let res = parse_llm_response(input).unwrap();
    assert!(res.command.contains("key"));
}

#[test]
fn test_parse_empty_string() {
    let res = parse_llm_response("");
    assert!(res.is_err());
}

#[test]
fn test_parse_no_json_object() {
    let input = "This is just plain text with no JSON at all.";
    let res = parse_llm_response(input);
    assert!(res.is_err());
}

#[test]
fn test_parse_malformed_json_unclosed_brace() {
    let input = r#"{"command": "ls", "explanation": "list files"#;
    let res = parse_llm_response(input);
    assert!(res.is_err());
}

#[test]
fn test_parse_extra_whitespace() {
    let input = r#"
    
    
    {  "command"  :  "ls -la"  ,  "explanation"  :  "list files"  ,  "dangerous"  :  false  }
    
    
    "#;
    let res = parse_llm_response(input).unwrap();
    assert_eq!(res.command, "ls -la");
}

#[test]
fn test_parse_unicode_command() {
    let input = r#"{"command": "echo 'Merhaba Dünya'", "explanation": "print hello in Turkish", "dangerous": false}"#;
    let res = parse_llm_response(input).unwrap();
    assert!(res.command.contains("Merhaba"));
}

#[test]
fn test_parse_escaped_backslashes() {
    let input = r#"{"command": "dir C:\\Users\\test", "explanation": "list directory", "dangerous": false}"#;
    let res = parse_llm_response(input).unwrap();
    assert!(res.command.contains("C:\\Users\\test"));
}

#[test]
fn test_parse_boolean_true_dangerous() {
    let input = r#"{"command": "dd if=/dev/zero of=/dev/sda", "explanation": "disk wipe", "dangerous": true}"#;
    let res = parse_llm_response(input).unwrap();
    assert!(res.dangerous);
}

#[test]
fn test_parse_multiple_json_objects() {
    let input = r#"{"command": "first", "explanation": "first cmd", "dangerous": false} {"command": "second", "explanation": "second cmd", "dangerous": true}"#;
    let res = parse_llm_response(input).unwrap();
    assert_eq!(res.command, "first");
    assert!(!res.dangerous);
}
