use anyhow::Result;
use std::env;
use std::process::Command;

use crate::session;
use crate::util::{project_root, ACTIVE_SESSION_FILE, STORAGE_DIR};

pub fn run() -> Result<()> {
    let root = project_root()?;
    let storage_dir = root.join(STORAGE_DIR);
    let active_session_file = root.join(ACTIVE_SESSION_FILE);

    println!("RunbookAI Doctor\n");
    println!("Project root: {}", root.display());

    print_check("Git", git_status());
    print_check(
        "Storage",
        if storage_dir.exists() {
            Check::Ok(format!("{} found", STORAGE_DIR))
        } else {
            Check::Warn(format!("{} not found; run `runbookai init`", STORAGE_DIR))
        },
    );

    print_check(
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
    );

    print_check("AI provider", ai_provider_status());
    print_check("Rust toolchain", rust_toolchain_status());
    print_check("Windows linker", windows_linker_status());

    Ok(())
}

enum Check {
    Ok(String),
    Warn(String),
    Error(String),
}

fn print_check(label: &str, check: Check) {
    match check {
        Check::Ok(message) => println!("{label}: OK - {message}"),
        Check::Warn(message) => println!("{label}: WARN - {message}"),
        Check::Error(message) => println!("{label}: ERROR - {message}"),
    }
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
            env::var("RUNBOOKAI_MODEL").unwrap_or_else(|_| "gpt-4o".to_string())
        ))
    } else if env::var("GEMINI_API_KEY").is_ok() {
        Check::Ok(format!(
            "Gemini ({})",
            env::var("RUNBOOKAI_MODEL").unwrap_or_else(|_| "gemini-1.5-pro".to_string())
        ))
    } else {
        Check::Ok(format!(
            "Ollama default ({})",
            env::var("RUNBOOKAI_MODEL").unwrap_or_else(|_| "llama3".to_string())
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
