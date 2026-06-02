use anyhow::{Context, Result};
use chrono::Utc;
use std::io::{self, Write};
use std::process::{Command, Stdio};
use std::time::Instant;
use uuid::Uuid;

use crate::config::load_config;
use crate::detect::detect_errors;
use crate::models::CommandRecord;
use crate::redact::maybe_redact;
use crate::session::{active_session_id, load_session, save_session};
use crate::util::preview;

pub fn exec_command(command: &str) -> Result<()> {
    let id =
        active_session_id().context("No active session. Run `runbookai start \"title\"` first.")?;
    let mut session = load_session(&id)?;
    let config = load_config()?;
    let cwd = std::env::current_dir()?;

    let started = Instant::now();
    let output = shell_command(command)
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .output()
        .with_context(|| format!("Failed to execute command: {command}"))?;
    let duration_ms = started.elapsed().as_millis();

    io::stdout().write_all(&output.stdout)?;
    io::stderr().write_all(&output.stderr)?;

    let stdout_raw = String::from_utf8_lossy(&output.stdout).to_string();
    let stderr_raw = String::from_utf8_lossy(&output.stderr).to_string();
    let stdout_clean = maybe_redact(&stdout_raw, config.redact_secrets)?;
    let stderr_clean = maybe_redact(&stderr_raw, config.redact_secrets)?;
    let stdout_preview = preview(&stdout_clean, config.max_output_length);
    let stderr_preview = preview(&stderr_clean, config.max_output_length);
    let mut detected_errors = detect_errors(&stdout_preview, "stdout")?;
    detected_errors.extend(detect_errors(&stderr_preview, "stderr")?);

    let record = CommandRecord {
        id: Uuid::new_v4().to_string(),
        timestamp: Utc::now(),
        cwd: cwd.display().to_string(),
        command: command.to_string(),
        exit_code: output.status.code().unwrap_or(-1),
        duration_ms,
        stdout_preview: if stdout_preview.is_empty() {
            None
        } else {
            Some(stdout_preview)
        },
        stderr_preview: if stderr_preview.is_empty() {
            None
        } else {
            Some(stderr_preview)
        },
        detected_errors,
    };

    session.commands.push(record);
    save_session(&session)?;

    if !output.status.success() {
        std::process::exit(output.status.code().unwrap_or(1));
    }

    Ok(())
}

pub fn note(kind: crate::models::NoteKind, content_parts: Vec<String>) -> Result<()> {
    let id =
        active_session_id().context("No active session. Run `runbookai start \"title\"` first.")?;
    let mut session = load_session(&id)?;
    let content = content_parts.join(" ").trim().to_string();
    if content.is_empty() {
        return Err(anyhow::anyhow!("Note content cannot be empty."));
    }

    session.notes.push(crate::models::NoteRecord {
        id: Uuid::new_v4().to_string(),
        timestamp: Utc::now(),
        kind,
        content: content.clone(),
    });
    save_session(&session)?;

    println!("Added {kind} note: {content}");
    Ok(())
}

fn shell_command(command: &str) -> Command {
    #[cfg(target_os = "windows")]
    {
        let mut cmd = Command::new("cmd");
        cmd.args(["/C", command]);
        cmd
    }

    #[cfg(not(target_os = "windows"))]
    {
        let mut cmd = Command::new("sh");
        cmd.args(["-c", command]);
        cmd
    }
}
