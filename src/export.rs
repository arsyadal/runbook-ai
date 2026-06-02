use anyhow::Result;
use std::fs;
use std::path::PathBuf;

use crate::cli::ExportFormat;
use crate::render::render_runbook;
use crate::session::latest_or_active_session;

pub fn export(format: ExportFormat, output: Option<PathBuf>) -> Result<()> {
    let session = latest_or_active_session()?;
    let content = match format {
        ExportFormat::Json => serde_json::to_string_pretty(&session)?,
        ExportFormat::Markdown => render_runbook(&session, None),
    };

    if let Some(path) = output {
        if let Some(parent) = path.parent() {
            if !parent.as_os_str().is_empty() {
                fs::create_dir_all(parent)?;
            }
        }
        fs::write(&path, content)?;
        println!("Exported {}", path.display());
    } else {
        println!("{content}");
    }
    Ok(())
}
