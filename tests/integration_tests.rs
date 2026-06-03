use assert_cmd::Command;
use std::fs;
use tempfile::TempDir;

fn runbookai() -> Command {
    Command::cargo_bin("runbookai").unwrap()
}

fn setup_repo() -> TempDir {
    let dir = TempDir::new().unwrap();
    let repo = dir.path();

    // Initialize a git repo so git snapshot works
    std::process::Command::new("git")
        .args(["init"])
        .current_dir(repo)
        .output()
        .expect("git init failed");

    // Set git user config so commits work if needed
    std::process::Command::new("git")
        .args(["config", "user.email", "test@test.com"])
        .current_dir(repo)
        .output()
        .unwrap();

    std::process::Command::new("git")
        .args(["config", "user.name", "Test User"])
        .current_dir(repo)
        .output()
        .unwrap();

    // Create a dummy file so git status has something to track
    fs::write(repo.join("README.md"), "# Test").unwrap();

    dir
}

#[test]
fn help_shows_usage() {
    runbookai()
        .arg("--help")
        .assert()
        .success()
        .stdout(predicates::str::contains("runbookai"))
        .stdout(predicates::str::contains("Turn AI coding agent sessions"));
}

#[test]
fn init_creates_storage_dirs() {
    let dir = setup_repo();
    let repo = dir.path();

    runbookai()
        .arg("init")
        .current_dir(repo)
        .assert()
        .success()
        .stdout(predicates::str::contains("RunbookAI initialized"));

    assert!(repo.join(".runbookai").exists());
    assert!(repo.join(".runbookai/config.json").exists());
    assert!(repo.join("docs/runbooks").exists());
}

#[test]
fn start_creates_active_session() {
    let dir = setup_repo();
    let repo = dir.path();

    runbookai().arg("init").current_dir(repo).assert().success();

    runbookai()
        .args(["start", "Fix login error"])
        .current_dir(repo)
        .assert()
        .success()
        .stdout(predicates::str::contains("RunbookAI recording started"))
        .stdout(predicates::str::contains("Session ID:"));

    assert!(repo.join(".runbookai/active_session").exists());
}

#[test]
fn status_shows_active_session() {
    let dir = setup_repo();
    let repo = dir.path();

    runbookai().arg("init").current_dir(repo).assert().success();
    runbookai()
        .args(["start", "Test session"])
        .current_dir(repo)
        .assert()
        .success();

    runbookai()
        .arg("status")
        .current_dir(repo)
        .assert()
        .success()
        .stdout(predicates::str::contains("Active RunbookAI session"))
        .stdout(predicates::str::contains("Test session"));
}

#[test]
fn exec_records_command() {
    let dir = setup_repo();
    let repo = dir.path();

    runbookai().arg("init").current_dir(repo).assert().success();
    runbookai()
        .args(["start", "Test session"])
        .current_dir(repo)
        .assert()
        .success();

    runbookai()
        .args(["exec", "echo hello"])
        .current_dir(repo)
        .assert()
        .success()
        .stdout(predicates::str::contains("hello"));
}

#[test]
fn exec_fails_on_bad_command() {
    let dir = setup_repo();
    let repo = dir.path();

    runbookai().arg("init").current_dir(repo).assert().success();
    runbookai()
        .args(["start", "Test session"])
        .current_dir(repo)
        .assert()
        .success();

    runbookai()
        .args(["exec", "exit 1"])
        .current_dir(repo)
        .assert()
        .failure();
}

#[test]
fn note_adds_to_session() {
    let dir = setup_repo();
    let repo = dir.path();

    runbookai().arg("init").current_dir(repo).assert().success();
    runbookai()
        .args(["start", "Test session"])
        .current_dir(repo)
        .assert()
        .success();

    runbookai()
        .args(["note", "--type", "root-cause", "Missing env var"])
        .current_dir(repo)
        .assert()
        .success()
        .stdout(predicates::str::contains("Added root-cause note"));
}

#[test]
fn stop_completes_session() {
    let dir = setup_repo();
    let repo = dir.path();

    runbookai().arg("init").current_dir(repo).assert().success();
    runbookai()
        .args(["start", "Test session"])
        .current_dir(repo)
        .assert()
        .success();

    runbookai()
        .arg("stop")
        .current_dir(repo)
        .assert()
        .success()
        .stdout(predicates::str::contains("RunbookAI recording stopped"))
        .stdout(predicates::str::contains("Session Summary:"));

    // Active session file should be removed
    assert!(!repo.join(".runbookai/active_session").exists());
}

#[test]
fn generate_runbook_creates_markdown() {
    let dir = setup_repo();
    let repo = dir.path();

    runbookai().arg("init").current_dir(repo).assert().success();
    runbookai()
        .args(["start", "Fix login error"])
        .current_dir(repo)
        .assert()
        .success();

    runbookai()
        .args(["exec", "echo hello"])
        .current_dir(repo)
        .assert()
        .success();

    runbookai()
        .args(["note", "--type", "root-cause", "Missing JWT secret"])
        .current_dir(repo)
        .assert()
        .success();

    runbookai().arg("stop").current_dir(repo).assert().success();

    runbookai()
        .args(["generate", "runbook"])
        .current_dir(repo)
        .assert()
        .success()
        .stdout(predicates::str::contains("Generated"));

    let docs_dir = repo.join("docs/runbooks");
    let entries: Vec<_> = fs::read_dir(&docs_dir)
        .unwrap()
        .filter_map(|e| e.ok())
        .collect();
    assert!(!entries.is_empty(), "No runbook generated");

    let runbook_file = &entries[0].path();
    let content = fs::read_to_string(runbook_file).unwrap();
    assert!(content.contains("# Runbook: Fix login error"));
    assert!(content.contains("echo hello"));
    assert!(content.contains("Missing JWT secret"));
}

#[test]
fn generate_all_creates_three_files() {
    let dir = setup_repo();
    let repo = dir.path();

    runbookai().arg("init").current_dir(repo).assert().success();
    runbookai()
        .args(["start", "Test session"])
        .current_dir(repo)
        .assert()
        .success();
    runbookai().arg("stop").current_dir(repo).assert().success();

    runbookai()
        .args(["generate", "all"])
        .current_dir(repo)
        .assert()
        .success();

    let docs_dir = repo.join("docs/runbooks");
    let entries: Vec<_> = fs::read_dir(&docs_dir)
        .unwrap()
        .filter_map(|e| e.ok())
        .collect();
    let md_files: Vec<_> = entries
        .iter()
        .filter(|e| e.path().extension().map(|ext| ext == "md").unwrap_or(false))
        .collect();

    assert_eq!(
        md_files.len(),
        4,
        "Expected 4 markdown files (runbook, changelog, postmortem, pr)"
    );
}

#[test]
fn export_json_outputs_valid_json() {
    let dir = setup_repo();
    let repo = dir.path();

    runbookai().arg("init").current_dir(repo).assert().success();
    runbookai()
        .args(["start", "Test session"])
        .current_dir(repo)
        .assert()
        .success();
    runbookai().arg("stop").current_dir(repo).assert().success();

    let output = runbookai()
        .args(["export", "--format", "json"])
        .current_dir(repo)
        .assert()
        .success()
        .get_output()
        .stdout
        .clone();

    let json_str = String::from_utf8(output).unwrap();
    let parsed: serde_json::Value = serde_json::from_str(&json_str).expect("Invalid JSON output");
    assert_eq!(parsed["title"], "Test session");
}

#[test]
fn double_start_fails() {
    let dir = setup_repo();
    let repo = dir.path();

    runbookai().arg("init").current_dir(repo).assert().success();
    runbookai()
        .args(["start", "First session"])
        .current_dir(repo)
        .assert()
        .success();

    runbookai()
        .args(["start", "Second session"])
        .current_dir(repo)
        .assert()
        .failure()
        .stderr(predicates::str::contains("already active"));
}

#[test]
fn status_without_session_prints_no_active() {
    let dir = setup_repo();
    let repo = dir.path();

    runbookai().arg("init").current_dir(repo).assert().success();

    runbookai()
        .arg("status")
        .current_dir(repo)
        .assert()
        .success()
        .stdout(predicates::str::contains("No active RunbookAI session"));
}
