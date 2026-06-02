use anyhow::{anyhow, Context, Result};
use chrono::Utc;
use std::fs;

use crate::git::git_snapshot;
use crate::models::{Config, Session, SessionStatus};
use crate::render::{changed_files, count_errors};
use crate::util::{project_root, write_json, ACTIVE_SESSION_FILE, STORAGE_DIR};
use uuid::Uuid;

pub fn init() -> Result<()> {
    let root = project_root()?;
    fs::create_dir_all(root.join(STORAGE_DIR).join("sessions"))?;
    fs::create_dir_all(root.join("docs/runbooks"))?;

    let config_path = root.join(STORAGE_DIR).join("config.json");
    if !config_path.exists() {
        let config = Config::default_for(&root);
        write_json(&config_path, &config)?;
    }

    println!("RunbookAI initialized.");
    println!("Storage: {}", root.join(STORAGE_DIR).display());
    println!("Docs:    {}", root.join("docs/runbooks").display());
    Ok(())
}

pub fn start(title_parts: Vec<String>) -> Result<()> {
    init_storage_if_needed()?;
    if active_session_id().is_ok() {
        return Err(anyhow!(
            "A RunbookAI session is already active. Run `runbookai status` or `runbookai stop`."
        ));
    }

    let root = project_root()?;
    let config = crate::config::load_config()?;
    let title = if title_parts.is_empty() {
        "Untitled Session".to_string()
    } else {
        title_parts.join(" ")
    };
    let id = format!(
        "rb_{}_{}",
        Utc::now().format("%Y_%m_%d_%H%M%S"),
        &Uuid::new_v4().to_string()[..8]
    );
    let snapshot = git_snapshot().ok();

    let session = Session {
        id: id.clone(),
        title,
        project_name: config.project_name,
        project_path: root.display().to_string(),
        branch_name: snapshot.as_ref().and_then(|s| s.branch.clone()),
        commit_hash: snapshot.as_ref().and_then(|s| s.commit_hash.clone()),
        started_at: Utc::now(),
        ended_at: None,
        status: SessionStatus::Active,
        commands: Vec::new(),
        notes: Vec::new(),
        git_before: snapshot,
        git_after: None,
    };

    save_session(&session)?;
    fs::write(root.join(ACTIVE_SESSION_FILE), &id)?;

    println!("RunbookAI recording started.\n");
    println!("Session ID: {}", session.id);
    println!("Project: {}", session.project_name);
    if let Some(branch) = &session.branch_name {
        println!("Branch: {branch}");
    }
    println!("\nRun commands with: runbookai exec \"<command>\"");
    println!("Add notes with:   runbookai note --type decision \"...\"");
    println!("Stop with:        runbookai stop");
    Ok(())
}

pub fn status() -> Result<()> {
    let id = match active_session_id() {
        Ok(id) => id,
        Err(_) => {
            println!("No active RunbookAI session.");
            return Ok(());
        }
    };
    let session = load_session(&id)?;
    let duration = Utc::now().signed_duration_since(session.started_at);

    println!("Active RunbookAI session\n");
    println!("Session ID: {}", session.id);
    println!("Title: {}", session.title);
    println!("Project: {}", session.project_name);
    if let Some(branch) = &session.branch_name {
        println!("Branch: {branch}");
    }
    println!("Duration: {} minutes", duration.num_minutes());
    println!("Commands: {}", session.commands.len());
    println!("Notes: {}", session.notes.len());
    println!("Errors: {}", count_errors(&session));
    Ok(())
}

pub fn stop() -> Result<()> {
    let id = active_session_id().context("No active session to stop.")?;
    let mut session = load_session(&id)?;
    session.ended_at = Some(Utc::now());
    session.status = SessionStatus::Completed;
    session.git_after = git_snapshot().ok();
    save_session(&session)?;
    let root = project_root()?;
    let _ = fs::remove_file(root.join(ACTIVE_SESSION_FILE));

    let duration = session
        .ended_at
        .unwrap_or_else(Utc::now)
        .signed_duration_since(session.started_at);
    println!("RunbookAI recording stopped.\n");
    println!("Session Summary:");
    println!("- Duration: {} minutes", duration.num_minutes());
    println!("- Commands recorded: {}", session.commands.len());
    println!("- Errors detected: {}", count_errors(&session));
    println!("- Files changed: {}", changed_files(&session).len());
    println!("- Notes added: {}", session.notes.len());
    println!("\nGenerate documentation:");
    println!("- runbookai generate runbook");
    println!("- runbookai generate changelog");
    println!("- runbookai generate postmortem");
    println!("- runbookai generate all");
    Ok(())
}

pub fn init_storage_if_needed() -> Result<()> {
    let root = project_root()?;
    if !root.join(STORAGE_DIR).exists() {
        init()?;
    }
    Ok(())
}

pub fn session_path(id: &str) -> Result<std::path::PathBuf> {
    Ok(project_root()?
        .join(STORAGE_DIR)
        .join("sessions")
        .join(id)
        .join("session.json"))
}

pub fn save_session(session: &Session) -> Result<()> {
    let dir = project_root()?
        .join(STORAGE_DIR)
        .join("sessions")
        .join(&session.id);
    fs::create_dir_all(&dir)?;
    write_json(&dir.join("session.json"), session)?;
    write_json(&dir.join("commands.json"), &session.commands)?;
    write_json(&dir.join("notes.json"), &session.notes)?;
    if let Some(git_before) = &session.git_before {
        write_json(&dir.join("git-before.json"), git_before)?;
    }
    if let Some(git_after) = &session.git_after {
        write_json(&dir.join("git-after.json"), git_after)?;
    }
    Ok(())
}

pub fn load_session(id: &str) -> Result<Session> {
    let raw = fs::read_to_string(session_path(id)?)?;
    Ok(serde_json::from_str(&raw)?)
}

pub fn active_session_id() -> Result<String> {
    let id = fs::read_to_string(project_root()?.join(ACTIVE_SESSION_FILE))?;
    Ok(id.trim().to_string())
}

pub fn latest_or_active_session() -> Result<Session> {
    if let Ok(id) = active_session_id() {
        return load_session(&id);
    }

    let sessions_dir = project_root()?.join(STORAGE_DIR).join("sessions");
    let mut entries = fs::read_dir(&sessions_dir)
        .with_context(|| "No RunbookAI sessions found. Run `runbookai start` first.")?
        .filter_map(|entry| entry.ok())
        .filter(|entry| entry.path().is_dir())
        .collect::<Vec<_>>();

    entries.sort_by_key(|entry| entry.file_name());
    let latest = entries
        .last()
        .ok_or_else(|| anyhow!("No RunbookAI sessions found. Run `runbookai start` first."))?;
    let id = latest.file_name().to_string_lossy().to_string();
    load_session(&id)
}
