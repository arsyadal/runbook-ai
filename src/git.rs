use anyhow::{anyhow, Result};
use std::collections::HashMap;
use std::process::Command;

use crate::models::{ChangedFile, GitSnapshot};

pub fn git_snapshot() -> Result<GitSnapshot> {
    Ok(GitSnapshot {
        branch: git_output(["rev-parse", "--abbrev-ref", "HEAD"]).ok(),
        commit_hash: git_output(["rev-parse", "HEAD"]).ok(),
        changed_files: parse_git_status()?,
    })
}

fn git_output<const N: usize>(args: [&str; N]) -> Result<String> {
    let output = Command::new("git").args(args).output()?;
    if !output.status.success() {
        return Err(anyhow!("git command failed"));
    }
    Ok(String::from_utf8_lossy(&output.stdout).trim().to_string())
}

fn parse_git_status() -> Result<Vec<ChangedFile>> {
    let output = Command::new("git").args(["status", "--short"]).output()?;
    if !output.status.success() {
        return Ok(Vec::new());
    }

    let raw = String::from_utf8_lossy(&output.stdout);
    let stats = diff_numstat().unwrap_or_default();
    let files = raw
        .lines()
        .filter(|line| line.len() >= 4)
        .map(|line| {
            let status_code = line[..2].trim().to_string();
            let path_raw = line[3..].trim();
            let path = path_raw
                .split(" -> ")
                .last()
                .unwrap_or(path_raw)
                .to_string();
            let status = match status_code.chars().next().unwrap_or('M') {
                'A' => "added",
                'D' => "deleted",
                'R' => "renamed",
                '?' => "added",
                _ => "modified",
            }
            .to_string();
            let (additions, deletions) = stats.get(&path).unwrap_or((None, None));
            ChangedFile {
                path,
                status,
                additions,
                deletions,
            }
        })
        .collect();
    Ok(files)
}

#[derive(Default, Clone)]
pub struct DiffStats {
    inner: HashMap<String, (Option<u32>, Option<u32>)>,
}

impl DiffStats {
    fn get(&self, path: &str) -> Option<(Option<u32>, Option<u32>)> {
        self.inner.get(path).copied()
    }
}

fn diff_numstat() -> Result<DiffStats> {
    let output = Command::new("git").args(["diff", "--numstat"]).output()?;
    if !output.status.success() {
        return Ok(DiffStats::default());
    }
    let mut inner = HashMap::new();
    for line in String::from_utf8_lossy(&output.stdout).lines() {
        let mut parts = line.splitn(3, '\t');
        let additions = parts.next().and_then(|v| v.parse::<u32>().ok());
        let deletions = parts.next().and_then(|v| v.parse::<u32>().ok());
        if let Some(path) = parts.next() {
            inner.insert(path.to_string(), (additions, deletions));
        }
    }
    Ok(DiffStats { inner })
}
