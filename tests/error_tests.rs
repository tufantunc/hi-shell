use hi_shell::error::{HiShellError, Result};

#[test]
fn test_error_display() {
    let err = HiShellError::Config("test message".to_string());
    assert_eq!(format!("{}", err), "Configuration error: test message");

    let api_err = HiShellError::Api {
        provider: "Gemini".to_string(),
        message: "Quota exceeded".to_string(),
    };
    assert_eq!(format!("{}", api_err), "API error (Gemini): Quota exceeded");
}

#[test]
fn test_error_conversion() {
    let io_err = std::io::Error::new(std::io::ErrorKind::NotFound, "file not found");
    let hi_err: HiShellError = io_err.into();
    match hi_err {
        HiShellError::Io(_) => (),
        _ => panic!("Expected Io error variant"),
    }
}

fn return_result() -> Result<()> {
    Err(HiShellError::Parsing("fail".to_string()))
}

#[test]
fn test_result_usage() {
    let res = return_result();
    assert!(res.is_err());
}
