use anyhow::Result;
use regex::Regex;

pub fn maybe_redact(input: &str, enabled: bool) -> Result<String> {
    if !enabled {
        return Ok(input.to_string());
    }
    redact_secrets(input)
}

pub fn redact_secrets(input: &str) -> Result<String> {
    let patterns = [
        r"(?i)(api[_-]?key\s*[=:]\s*)\S+",
        r"(?i)(access[_-]?token\s*[=:]\s*)\S+",
        r"(?i)(bearer\s+)[A-Za-z0-9._\-]+",
        r"(?i)(password\s*[=:]\s*)\S+",
        r"(?i)(secret\s*[=:]\s*)\S+",
        r"(?i)(jwt\s*[=:]\s*)\S+",
        r"(?i)(database_url\s*[=:]\s*)\S+",
        r"postgres://[^\s]+",
        r"mysql://[^\s]+",
        r"mongodb(\+srv)?://[^\s]+",
        r"-----BEGIN [A-Z ]*PRIVATE KEY-----[\s\S]*?-----END [A-Z ]*PRIVATE KEY-----",
    ];

    let mut output = input.to_string();
    for pattern in patterns {
        let re = Regex::new(pattern)?;
        output = re.replace_all(&output, "$1[REDACTED]").to_string();
    }
    Ok(output)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn redacts_api_key() {
        let output = redact_secrets("API_KEY=super-secret").unwrap();
        assert!(output.contains("[REDACTED]"));
        assert!(!output.contains("super-secret"));
    }

    #[test]
    fn redacts_bearer_token() {
        let output =
            redact_secrets("Authorization: bearer eyJhbGciOiJIUzI1NiIsInR5cCI6IkpXVCJ9").unwrap();
        assert!(output.contains("bearer [REDACTED]"));
    }

    #[test]
    fn redacts_private_key() {
        let input = "-----BEGIN PRIVATE KEY-----\nabc123\n-----END PRIVATE KEY-----";
        let output = redact_secrets(input).unwrap();
        assert!(output.contains("[REDACTED]"));
        assert!(!output.contains("abc123"));
    }

    #[test]
    fn maybe_redact_disabled_passes_through() {
        let input = "API_KEY=secret";
        let output = maybe_redact(input, false).unwrap();
        assert_eq!(output, input);
    }

    #[test]
    fn redacts_database_url() {
        let output = redact_secrets("DATABASE_URL=postgres://user:pass@localhost/db").unwrap();
        assert!(!output.contains("postgres://user:pass"));
        assert!(output.contains("[REDACTED]"));
    }
}
