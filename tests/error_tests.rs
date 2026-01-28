use sdiff_rs::{OutputError, ParseError, SdiffError};

#[test]
fn test_parse_error_display() {
    let err = ParseError::file_not_found("test.json");
    assert_eq!(err.to_string(), "File not found: test.json");
}

#[test]
fn test_unknown_format_error() {
    let err = ParseError::unknown_format("/path/to/file.txt");
    assert!(err.to_string().contains("Could not detect file format"));
    assert!(err.to_string().contains("/path/to/file.txt"));
}

#[test]
fn test_output_error_display() {
    let err = OutputError::UnknownFormat {
        format: "xml".to_string(),
    };
    assert_eq!(err.to_string(), "Unknown output format: xml");
}

#[test]
fn test_sdiff_error_from_parse_error() {
    let parse_err = ParseError::file_not_found("test.json");
    let sdiff_err: SdiffError = parse_err.into();
    assert!(matches!(sdiff_err, SdiffError::Parse(_)));
}

#[test]
fn test_config_error() {
    let err = SdiffError::ConfigError {
        message: "Invalid option".to_string(),
    };
    assert!(err.to_string().contains("Invalid configuration"));
}
