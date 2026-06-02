use anyhow::Result;

use crate::models::DetectedError;

pub fn detect_errors(input: &str, source: &str) -> Result<Vec<DetectedError>> {
    let patterns = [
        "Error",
        "Exception",
        "TypeError",
        "ReferenceError",
        "SyntaxError",
        "AssertionError",
        "Compilation failed",
        "Build failed",
        "Test failed",
        "Connection refused",
        "Timeout",
        "Permission denied",
        "Module not found",
        "Cannot find module",
        "Port already in use",
        "Migration failed",
    ];
    let mut errors = Vec::new();
    for line in input.lines() {
        for pattern in patterns {
            if line.to_lowercase().contains(&pattern.to_lowercase()) {
                errors.push(DetectedError {
                    kind: pattern.to_string(),
                    message: line.trim().chars().take(240).collect(),
                    source: source.to_string(),
                    severity: if pattern.contains("failed") || pattern.contains("Error") {
                        "high".to_string()
                    } else {
                        "medium".to_string()
                    },
                });
                break;
            }
        }
    }
    Ok(errors)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn detects_compilation_failed() {
        let errors = detect_errors("Compilation failed: missing module", "stderr").unwrap();
        assert_eq!(errors.len(), 1);
        assert_eq!(errors[0].kind, "Compilation failed");
        assert_eq!(errors[0].source, "stderr");
        assert_eq!(errors[0].severity, "high");
    }

    #[test]
    fn detects_multiple_different_errors() {
        let input = "Error: something broke\nAnother line\nTimeout connecting to server";
        let errors = detect_errors(input, "stdout").unwrap();
        assert_eq!(errors.len(), 2);
        assert_eq!(errors[0].kind, "Error");
        assert_eq!(errors[1].kind, "Timeout");
    }

    #[test]
    fn detects_permission_denied_medium_severity() {
        let errors = detect_errors("Permission denied: /etc/shadow", "stderr").unwrap();
        assert_eq!(errors.len(), 1);
        assert_eq!(errors[0].kind, "Permission denied");
        assert_eq!(errors[0].severity, "medium");
    }

    #[test]
    fn no_false_positives_on_clean_output() {
        let errors = detect_errors("Everything is working fine", "stdout").unwrap();
        assert!(errors.is_empty());
    }

    #[test]
    fn message_truncated_to_240_chars() {
        let long_message = "E".repeat(300);
        let input = format!("Error: {long_message}");
        let errors = detect_errors(&input, "stdout").unwrap();
        assert_eq!(errors[0].message.len(), 240);
    }
}
