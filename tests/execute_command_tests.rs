use hi_shell::command::{execute_command, truncate_output};

#[test]
fn test_execute_simple_command() {
    let (output, success) = execute_command("echo hello").unwrap();
    assert!(success);
    assert!(output.contains("hello"));
}

#[test]
fn test_execute_failing_command() {
    let (_output, success) = execute_command("false").unwrap();
    assert!(!success);
}

#[test]
fn test_execute_command_with_stderr() {
    let (output, success) = execute_command("echo errormsg >&2").unwrap();
    assert!(success);
    assert!(output.contains("errormsg"));
}

#[test]
fn test_execute_command_combined_output() {
    let (output, success) = execute_command("echo out && echo err >&2").unwrap();
    assert!(success);
    assert!(output.contains("out"));
    assert!(output.contains("err"));
}

#[test]
fn test_execute_pipe_command() {
    let (output, success) = execute_command("echo hello | wc -c").unwrap();
    assert!(success);
    assert!(!output.trim().is_empty());
}

#[test]
fn test_execute_unicode_output() {
    let (output, success) = execute_command("echo 'Merhaba Dünya 🌍'").unwrap();
    assert!(success);
    assert!(output.contains("Merhaba"));
}

#[test]
fn test_truncate_output_short() {
    let output = "short string";
    let truncated = truncate_output(output, 1000);
    assert_eq!(truncated, "short string");
}

#[test]
fn test_truncate_output_exact_boundary() {
    let output = "a".repeat(1000);
    let truncated = truncate_output(&output, 1000);
    assert_eq!(truncated, output);
}

#[test]
fn test_truncate_output_long() {
    let output = "a".repeat(2000);
    let truncated = truncate_output(&output, 1000);
    assert!(truncated.ends_with("... (truncated)"));
    assert!(truncated.len() < 1100);
}

#[test]
fn test_truncate_output_utf8_multibyte() {
    let output = "ä".repeat(500);
    assert!(output.len() > 500);

    let truncated = truncate_output(&output, 500);
    assert!(truncated.ends_with("... (truncated)"));

    let valid = std::str::from_utf8(truncated.as_bytes());
    assert!(valid.is_ok());
}

#[test]
fn test_truncate_output_utf8_at_boundary() {
    let mut output = String::new();
    while output.len() < 999 {
        output.push('a');
    }
    output.push('ö');
    output.push('ö');

    assert!(output.len() > 1000);

    let truncated = truncate_output(&output, 1000);
    let valid = std::str::from_utf8(truncated.as_bytes());
    assert!(valid.is_ok());
}

#[test]
fn test_truncate_output_emoji() {
    let output = "🌍".repeat(300);
    assert!(output.len() > 1000);

    let truncated = truncate_output(&output, 1000);
    let valid = std::str::from_utf8(truncated.as_bytes());
    assert!(valid.is_ok());
}
