use anyhow::Result;
use clap::Parser;

mod cli;
mod command;
mod config;
mod detect;
mod export;
mod git;
mod models;
mod redact;
mod render;
mod session;
mod util;

use cli::{Cli, Commands, GenerateTarget};

fn main() -> Result<()> {
    let cli = Cli::parse();

    match cli.command {
        Commands::Init => session::init(),
        Commands::Start { title } => session::start(title),
        Commands::Status => session::status(),
        Commands::Exec { command } => command::exec_command(&command),
        Commands::Note { kind, content } => command::note(kind, content),
        Commands::Stop => session::stop(),
        Commands::Generate { target } => generate(target),
        Commands::Export { format, output } => export::export(format, output),
    }
}

fn generate(target: GenerateTarget) -> Result<()> {
    let s = session::latest_or_active_session()?;
    let cfg = config::load_config()?;
    let out_dir = util::project_root()?.join(cfg.output_dir);
    std::fs::create_dir_all(&out_dir)?;
    let slug = render::session_slug(&s);

    match target {
        GenerateTarget::Runbook => {
            let path = out_dir.join(format!("{slug}.md"));
            std::fs::write(&path, render::render_runbook(&s))?;
            println!("Generated {}", path.display());
        }
        GenerateTarget::Changelog => {
            let path = out_dir.join(format!("{slug}.changelog.md"));
            std::fs::write(&path, render::render_changelog(&s))?;
            println!("Generated {}", path.display());
        }
        GenerateTarget::Postmortem => {
            let path = out_dir.join(format!("{slug}.postmortem.md"));
            std::fs::write(&path, render::render_postmortem(&s))?;
            println!("Generated {}", path.display());
        }
        GenerateTarget::All => {
            let runbook = out_dir.join(format!("{slug}.md"));
            let changelog = out_dir.join(format!("{slug}.changelog.md"));
            let postmortem = out_dir.join(format!("{slug}.postmortem.md"));
            std::fs::write(&runbook, render::render_runbook(&s))?;
            std::fs::write(&changelog, render::render_changelog(&s))?;
            std::fs::write(&postmortem, render::render_postmortem(&s))?;
            println!("Generated {}", runbook.display());
            println!("Generated {}", changelog.display());
            println!("Generated {}", postmortem.display());
        }
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn redacts_common_secrets() {
        let input =
            "API_KEY=abc123 password=hunter2 DATABASE_URL=postgres://user:pass@localhost/db";
        let output = redact::redact_secrets(input).unwrap();

        assert!(!output.contains("abc123"));
        assert!(!output.contains("hunter2"));
        assert!(!output.contains("postgres://user:pass"));
        assert!(output.contains("[REDACTED]"));
    }

    #[test]
    fn detects_error_lines() {
        let errors = detect::detect_errors("Compilation failed: missing module", "stderr").unwrap();

        assert_eq!(errors.len(), 1);
        assert_eq!(errors[0].kind, "Compilation failed");
        assert_eq!(errors[0].source, "stderr");
    }
}
