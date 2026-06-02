use anyhow::Result;
use serde::Serialize;
use std::fs;
use std::path::{Path, PathBuf};

pub const STORAGE_DIR: &str = ".runbookai";
pub const ACTIVE_SESSION_FILE: &str = ".runbookai/active_session";

pub fn write_json<T: Serialize>(path: &Path, value: &T) -> Result<()> {
    if let Some(parent) = path.parent() {
        fs::create_dir_all(parent)?;
    }
    fs::write(path, serde_json::to_string_pretty(value)?)?;
    Ok(())
}

pub fn project_root() -> Result<PathBuf> {
    let cwd = std::env::current_dir()?;
    if let Ok(output) = std::process::Command::new("git")
        .args(["rev-parse", "--show-toplevel"])
        .output()
    {
        if output.status.success() {
            let root = String::from_utf8_lossy(&output.stdout).trim().to_string();
            if !root.is_empty() {
                return Ok(PathBuf::from(root));
            }
        }
    }
    Ok(cwd)
}

pub fn preview(input: &str, max_len: usize) -> String {
    if input.len() <= max_len {
        return input.to_string();
    }
    format!("{}\n...[truncated to {} bytes]", &input[..max_len], max_len)
}

pub fn markdown_list_or_placeholder(items: Vec<String>, placeholder: &str) -> String {
    if items.is_empty() {
        return placeholder.to_string();
    }
    items
        .into_iter()
        .map(|item| {
            if item.starts_with("- ") {
                item
            } else {
                format!("- {item}")
            }
        })
        .collect::<Vec<_>>()
        .join("\n")
}

pub fn non_empty<'a>(value: &'a str, fallback: &'a str) -> &'a str {
    if value.trim().is_empty() {
        fallback
    } else {
        value
    }
}

pub fn local_time(time: chrono::DateTime<chrono::Utc>) -> String {
    use chrono::Local;
    time.with_timezone(&Local)
        .format("%Y-%m-%d %H:%M:%S %Z")
        .to_string()
}

pub fn escape_table(value: &str) -> String {
    value.replace('|', "\\|").replace('\n', " ")
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn preview_short_input_unchanged() {
        let input = "short";
        assert_eq!(preview(input, 100), "short");
    }

    #[test]
    fn preview_long_input_truncated() {
        let input = "a".repeat(200);
        let result = preview(&input, 100);
        assert!(result.contains("...[truncated to 100 bytes]"));
        assert!(result.starts_with(&"a".repeat(100)));
    }

    #[test]
    fn markdown_list_or_placeholder_empty() {
        assert_eq!(
            markdown_list_or_placeholder(vec![], "placeholder"),
            "placeholder"
        );
    }

    #[test]
    fn markdown_list_or_placeholder_items() {
        let items = vec!["item one".to_string(), "- item two".to_string()];
        let result = markdown_list_or_placeholder(items, "placeholder");
        assert_eq!(result, "- item one\n- item two");
    }

    #[test]
    fn non_empty_returns_value_when_present() {
        assert_eq!(non_empty("hello", "fallback"), "hello");
    }

    #[test]
    fn non_empty_returns_fallback_when_empty() {
        assert_eq!(non_empty("   ", "fallback"), "fallback");
    }

    #[test]
    fn escape_table_replaces_pipes_and_newlines() {
        assert_eq!(escape_table("a|b\nc"), "a\\|b c");
    }
}
