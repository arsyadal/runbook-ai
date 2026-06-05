use anyhow::Result;
use serde::Serialize;
use std::env;
use std::process::Command;

use crate::session;
use crate::util::{project_root, ACTIVE_SESSION_FILE, STORAGE_DIR};

pub fn run(json: bool) -> Result<()> {
    let report = build_report()?;

    if json {
        println!("{}", serde_json::to_string_pretty(&report)?);
    } else {
        print_text_report(&report);
    }

    Ok(())
}

#[derive(Serialize)]
struct DoctorReport {
    project_root: String,
    checks: Vec<DoctorCheck>,
    summary: DoctorSummary,
}

#[derive(Serialize)]
struct DoctorCheck {
    label: String,
    status: CheckStatus,
    message: String,
}

#[derive(Clone, Copy, Serialize)]
#[serde(rename_all = "lowercase")]
enum CheckStatus {
    Pass,
    Warn,
    Fail,
}

#[derive(Serialize)]
struct DoctorSummary {
    pass: usize,
    warn: usize,
    fail: usize,
}

enum Check {
    Ok(String),
    Warn(String),
    Error(String),
}

fn build_report() -> Result<DoctorReport> {
    let root = project_root()?;
    let storage_dir = root.join(STORAGE_DIR);
    let active_session_file = root.join(ACTIVE_SESSION_FILE);

    let checks = vec![
        to_doctor_check("Git", git_status()),
        to_doctor_check(
            "Storage",
            if storage_dir.exists() {
                Check::Ok(format!("{} found", STORAGE_DIR))
            } else {
                Check::Warn(format!("{} not found; run `runbook init`", STORAGE_DIR))
            },
        ),
        to_doctor_check(
            "Active session",
            if active_session_file.exists() {
                match session::active_session_id() {
                    Ok(id) => Check::Ok(id),
                    Err(err) => Check::Warn(format!(
                        "active session file exists but is unreadable: {err}"
                    )),
                }
            } else {
                Check::Ok("none".to_string())
            },
        ),
        to_doctor_check("AI provider", ai_provider_status()),
        to_doctor_check("Rust toolchain", rust_toolchain_status()),
        to_doctor_check("Windows linker", windows_linker_status()),
    ];

    let summary = summarize(&checks);

    Ok(DoctorReport {
        project_root: root.display().to_string(),
        checks,
        summary,
    })
}

fn to_doctor_check(label: &str, check: Check) -> DoctorCheck {
    match check {
        Check::Ok(message) => DoctorCheck {
            label: label.to_string(),
            status: CheckStatus::Pass,
            message,
        },
        Check::Warn(message) => DoctorCheck {
            label: label.to_string(),
            status: CheckStatus::Warn,
            message,
        },
        Check::Error(message) => DoctorCheck {
            label: label.to_string(),
            status: CheckStatus::Fail,
            message,
        },
    }
}

fn summarize(checks: &[DoctorCheck]) -> DoctorSummary {
    DoctorSummary {
        pass: checks
            .iter()
            .filter(|check| matches!(check.status, CheckStatus::Pass))
            .count(),
        warn: checks
            .iter()
            .filter(|check| matches!(check.status, CheckStatus::Warn))
            .count(),
        fail: checks
            .iter()
            .filter(|check| matches!(check.status, CheckStatus::Fail))
            .count(),
    }
}

fn print_text_report(report: &DoctorReport) {
    println!("Runbook Doctor\n");
    println!("Project root: {}", report.project_root);
    for check in &report.checks {
        let status = match check.status {
            CheckStatus::Pass => "PASS",
            CheckStatus::Warn => "WARN",
            CheckStatus::Fail => "FAIL",
        };
        println!("{}: {} - {}", check.label, status, check.message);
    }
    println!(
        "\nSummary: {} pass, {} warn, {} fail",
        report.summary.pass, report.summary.warn, report.summary.fail
    );
}

fn git_status() -> Check {
    match Command::new("git").arg("--version").output() {
        Ok(output) if output.status.success() => {
            let version = String::from_utf8_lossy(&output.stdout).trim().to_string();
            Check::Ok(version)
        }
        Ok(output) => Check::Error(format!("git exited with {}", output.status)),
        Err(err) => Check::Error(format!("git not available: {err}")),
    }
}

fn ai_provider_status() -> Check {
    if env::var("OPENAI_API_KEY").is_ok() {
        Check::Ok(format!(
            "OpenAI ({})",
            env::var("RUNBOOK_MODEL").unwrap_or_else(|_| "gpt-4o".to_string())
        ))
    } else if env::var("GEMINI_API_KEY").is_ok() {
        Check::Ok(format!(
            "Gemini ({})",
            env::var("RUNBOOK_MODEL").unwrap_or_else(|_| "gemini-1.5-pro".to_string())
        ))
    } else {
        Check::Ok(format!(
            "Ollama default ({})",
            env::var("RUNBOOK_MODEL").unwrap_or_else(|_| "llama3".to_string())
        ))
    }
}

fn rust_toolchain_status() -> Check {
    match Command::new("rustc").arg("-vV").output() {
        Ok(output) if output.status.success() => {
            let stdout = String::from_utf8_lossy(&output.stdout);
            let host = stdout
                .lines()
                .find_map(|line| line.strip_prefix("host: "))
                .unwrap_or("unknown");
            Check::Ok(host.to_string())
        }
        Ok(output) => Check::Warn(format!("rustc exited with {}", output.status)),
        Err(err) => Check::Warn(format!("rustc not available: {err}")),
    }
}

#[cfg(windows)]
fn windows_linker_status() -> Check {
    let link_path = Command::new("where.exe")
        .arg("link")
        .output()
        .ok()
        .and_then(|output| {
            if output.status.success() {
                String::from_utf8(output.stdout).ok()
            } else {
                None
            }
        })
        .and_then(|stdout| stdout.lines().next().map(str::to_string));

    if let Some(path) = link_path {
        if path.contains("Git\\usr\\bin\\link.exe") || path.contains("Git/usr/bin/link.exe") {
            Check::Warn(format!(
                "Git Bash link.exe detected at {path}; use Developer PowerShell for MSVC"
            ))
        } else {
            Check::Ok(path)
        }
    } else {
        Check::Warn("link.exe not found on PATH".to_string())
    }
}

#[cfg(not(windows))]
fn windows_linker_status() -> Check {
    Check::Ok("not applicable".to_string())
}
