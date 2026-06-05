use anyhow::Result;
use clap::Parser;

mod ai;
mod cli;
mod command;
mod config;
mod detect;
mod doctor;
mod export;
mod git;
mod mcp;
mod models;
mod redact;
mod render;
mod session;
mod util;

use cli::{Cli, Commands, GenerateTarget, McpSubcommand};

#[tokio::main]
async fn main() -> Result<()> {
    let cli = Cli::parse();

    match cli.command {
        Commands::Init => session::init(),
        Commands::Start { title } => session::start(title),
        Commands::Status => session::status(),
        Commands::Doctor { json } => doctor::run(json),
        Commands::Exec { command } => command::exec_command(&command),
        Commands::Note { kind, content } => command::note(kind, content),
        Commands::Stop => session::stop(),
        Commands::Generate { target, ai } => generate(target, ai).await,
        Commands::Export { format, output } => export::export(format, output),
        Commands::Search { query } => search(&query),
        Commands::Alias => print_aliases(),
        Commands::ShellHook { shell } => print_shell_hook(&shell),
        Commands::Record {
            command,
            exit_code,
            duration,
            stdout,
            stderr,
        } => command::record_headless(command, exit_code, duration, stdout, stderr),
        Commands::Mcp { subcommand } => match subcommand {
            McpSubcommand::Serve => mcp::run_server().await,
        },
    }
}

fn print_shell_hook(shell: &str) -> Result<()> {
    match shell {
        "zsh" => {
            println!(
                r#"# Runbook Zsh Integration
_runbook_preexec() {{
    export _RB_START_TIME=$(date +%s%3N)
    export _RB_LAST_CMD="$1"
}}
_runbook_precmd() {{
    local exit_code=$?
    if [ -n "$_RB_LAST_CMD" ]; then
        local end_time=$(date +%s%3N)
        local duration=$((end_time - _RB_START_TIME))
        # Headless record in background
        runbook record --command "$_RB_LAST_CMD" --exit-code $exit_code --duration $duration > /dev/null 2>&1 &!
        unset _RB_LAST_CMD
    fi
}}
autoload -Uz add-zsh-hook
add-zsh-hook preexec _runbook_preexec
add-zsh-hook precmd _runbook_precmd
"#
            );
        }
        "bash" => {
            println!(
                r#"# Runbook Bash Integration
_runbook_bash_hook() {{
    local exit_code=$?
    if [ -n "$_RB_LAST_CMD" ]; then
        local end_time=$(date +%s%3N)
        local duration=$((end_time - _RB_START_TIME))
        runbook record --command "$_RB_LAST_CMD" --exit-code $exit_code --duration $duration > /dev/null 2>&1 &
        unset _RB_LAST_CMD
    fi
}}
trap 'export _RB_START_TIME=$(date +%s%3N); export _RB_LAST_CMD="$BASH_COMMAND"' DEBUG
PROMPT_COMMAND="_runbook_bash_hook; $PROMPT_COMMAND"
"#
            );
        }
        "powershell" | "pwsh" => {
            println!(
                r#"# Runbook PowerShell Integration
$global:RunbookLastHistoryId = $null

function global:prompt {{
    $exitCode = if ($LASTEXITCODE -eq $null) {{ 0 }} else {{ $LASTEXITCODE }}
    $history = Get-History -Count 1
    if ($history -and $history.Id -ne $global:RunbookLastHistoryId) {{
        $global:RunbookLastHistoryId = $history.Id
        Start-Process runbook -ArgumentList @(
            'record',
            '--command', $history.CommandLine,
            '--exit-code', $exitCode,
            '--duration', 0
        ) -NoNewWindow -Wait | Out-Null
    }}
    "PS $($executionContext.SessionState.Path.CurrentLocation)$('>' * ($nestedPromptLevel + 1)) "
}}
"#
            );
        }
        _ => {
            return Err(anyhow::anyhow!(
                "Unsupported shell: {}. Use 'zsh', 'bash', or 'powershell'.",
                shell
            ))
        }
    }
    Ok(())
}

fn print_aliases() -> Result<()> {
    println!("# Add this to your .bashrc or .zshrc:");
    println!("alias rb='runbook'");
    println!("alias rbx='runbook exec'");
    println!("alias rbn='runbook note'");
    println!("alias rbs='runbook status'");
    Ok(())
}

fn search(query: &str) -> Result<()> {
    let results = session::search_sessions(query)?;
    if results.is_empty() {
        println!("No sessions found matching: {}", query);
    } else {
        println!("Found {} matching sessions:\n", results.len());
        for session in results {
            println!("- {} (ID: {})", session.title, session.id);
            println!("  Started: {}", util::local_time(session.started_at));
            println!(
                "  Commands: {}, Notes: {}",
                session.commands.len(),
                session.notes.len()
            );
            println!();
        }
    }
    Ok(())
}

async fn generate(target: GenerateTarget, ai: bool) -> Result<()> {
    let s = session::latest_or_active_session()?;
    let cfg = config::load_config()?;
    let out_dir = util::project_root()?.join(cfg.output_dir);
    std::fs::create_dir_all(&out_dir)?;
    let slug = render::session_slug(&s);

    let mut ai_summary = None;
    if ai {
        println!(
            "Generating AI summary... (using model: {})",
            ai::AIService::from_env().model
        );
        let ai_service = ai::AIService::from_env();
        let session_json = serde_json::to_string_pretty(&s)?;
        match ai_service.summarize(&session_json).await {
            Ok(summary) => ai_summary = Some(summary),
            Err(e) => eprintln!(
                "AI Summary failed: {}. Continuing with default template.",
                e
            ),
        }
    }

    match target {
        GenerateTarget::Runbook => {
            let path = out_dir.join(format!("{slug}.md"));
            std::fs::write(&path, render::render_runbook(&s, ai_summary.as_deref()))?;
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
        GenerateTarget::Pr => {
            let path = out_dir.join(format!("{slug}.pr.md"));
            std::fs::write(&path, render::render_pr(&s, ai_summary.as_deref()))?;
            println!("Generated {}", path.display());
        }
        GenerateTarget::All => {
            let runbook = out_dir.join(format!("{slug}.md"));
            let changelog = out_dir.join(format!("{slug}.changelog.md"));
            let postmortem = out_dir.join(format!("{slug}.postmortem.md"));
            let pr = out_dir.join(format!("{slug}.pr.md"));
            std::fs::write(&runbook, render::render_runbook(&s, ai_summary.as_deref()))?;
            std::fs::write(&changelog, render::render_changelog(&s))?;
            std::fs::write(&postmortem, render::render_postmortem(&s))?;
            std::fs::write(&pr, render::render_pr(&s, ai_summary.as_deref()))?;
            println!("Generated {}", runbook.display());
            println!("Generated {}", changelog.display());
            println!("Generated {}", postmortem.display());
            println!("Generated {}", pr.display());
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
