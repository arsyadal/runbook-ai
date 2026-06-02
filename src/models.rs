use chrono::{DateTime, Utc};
use clap::ValueEnum;
use serde::{Deserialize, Serialize};

#[derive(Debug, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct Config {
    pub project_name: String,
    pub output_dir: String,
    pub track_git_diff: bool,
    pub track_command_output: bool,
    pub redact_secrets: bool,
    pub max_output_length: usize,
}

impl Config {
    pub fn default_for(project_root: &std::path::Path) -> Self {
        let project_name = project_root
            .file_name()
            .and_then(|name| name.to_str())
            .unwrap_or("project")
            .to_string();

        Self {
            project_name,
            output_dir: "docs/runbooks".to_string(),
            track_git_diff: true,
            track_command_output: true,
            redact_secrets: true,
            max_output_length: 5000,
        }
    }
}

#[derive(Debug, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct Session {
    pub id: String,
    pub title: String,
    pub project_name: String,
    pub project_path: String,
    pub branch_name: Option<String>,
    pub commit_hash: Option<String>,
    pub started_at: DateTime<Utc>,
    pub ended_at: Option<DateTime<Utc>>,
    pub status: SessionStatus,
    pub commands: Vec<CommandRecord>,
    pub notes: Vec<NoteRecord>,
    pub git_before: Option<GitSnapshot>,
    pub git_after: Option<GitSnapshot>,
}

#[derive(Debug, Deserialize, Serialize)]
#[serde(rename_all = "kebab-case")]
pub enum SessionStatus {
    Active,
    Completed,
}

#[derive(Debug, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct CommandRecord {
    pub id: String,
    pub timestamp: DateTime<Utc>,
    pub cwd: String,
    pub command: String,
    pub exit_code: i32,
    pub duration_ms: u128,
    pub stdout_preview: Option<String>,
    pub stderr_preview: Option<String>,
    pub detected_errors: Vec<DetectedError>,
}

#[derive(Clone, Copy, Debug, Deserialize, Serialize, ValueEnum)]
#[serde(rename_all = "kebab-case")]
pub enum NoteKind {
    Decision,
    Finding,
    Todo,
    Risk,
    Workaround,
    RootCause,
}

impl std::fmt::Display for NoteKind {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let value = match self {
            NoteKind::Decision => "decision",
            NoteKind::Finding => "finding",
            NoteKind::Todo => "todo",
            NoteKind::Risk => "risk",
            NoteKind::Workaround => "workaround",
            NoteKind::RootCause => "root-cause",
        };
        write!(f, "{value}")
    }
}

#[derive(Debug, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct NoteRecord {
    pub id: String,
    pub timestamp: DateTime<Utc>,
    pub kind: NoteKind,
    pub content: String,
}

#[derive(Debug, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct GitSnapshot {
    pub branch: Option<String>,
    pub commit_hash: Option<String>,
    pub changed_files: Vec<ChangedFile>,
    pub diff_content: Option<String>,
}

#[derive(Debug, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct ChangedFile {
    pub path: String,
    pub status: String,
    pub additions: Option<u32>,
    pub deletions: Option<u32>,
}

#[derive(Debug, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct DetectedError {
    pub kind: String,
    pub message: String,
    pub source: String,
    pub severity: String,
}
