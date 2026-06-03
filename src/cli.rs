use crate::models::NoteKind;
use clap::{Parser, Subcommand, ValueEnum};
use std::path::PathBuf;

#[derive(Parser)]
#[command(name = "runbookai")]
#[command(about = "Turn AI coding agent sessions into reusable runbooks.")]
#[command(version)]
pub struct Cli {
    #[command(subcommand)]
    pub command: Commands,
}

#[derive(Subcommand)]
pub enum Commands {
    /// Initialize RunbookAI storage in the current project.
    Init,
    /// Start a new recording session.
    Start {
        /// Short session title.
        title: Vec<String>,
    },
    /// Show active session status.
    Status,
    /// Diagnose local RunbookAI setup and environment.
    Doctor,
    /// Execute and record a command.
    Exec {
        /// Command to run, for example: "npm test".
        command: String,
    },
    /// Add a manual note to the active session.
    Note {
        /// Note type.
        #[arg(long = "type", value_enum, default_value_t = NoteKind::Finding)]
        kind: NoteKind,
        /// Note content.
        content: Vec<String>,
    },
    /// Stop the active recording session.
    Stop,
    /// Generate documentation.
    Generate {
        #[command(subcommand)]
        target: GenerateTarget,
        /// Use AI to generate a session summary (requires Ollama, OpenAI, or Gemini).
        #[arg(long)]
        ai: bool,
    },
    /// Export session data.
    Export {
        /// Export format.
        #[arg(long, value_enum, default_value_t = ExportFormat::Json)]
        format: ExportFormat,
        /// Output file path. Prints to stdout when omitted.
        #[arg(long)]
        output: Option<PathBuf>,
    },
    /// Search through old sessions.
    Search {
        /// Query to search for.
        query: String,
    },
    /// Print shell alias definitions.
    Alias,
    /// Generate shell hook script for automatic recording.
    ShellHook {
        /// Shell type (zsh, bash).
        #[arg(default_value = "zsh")]
        shell: String,
    },
    /// Record a command manually (headless).
    Record {
        /// The command that was executed.
        #[arg(long)]
        command: String,
        /// Exit code of the command.
        #[arg(long)]
        exit_code: i32,
        /// Duration of the command in milliseconds.
        #[arg(long)]
        duration: u128,
        /// Captured stdout (optional).
        #[arg(long)]
        stdout: Option<String>,
        /// Captured stderr (optional).
        #[arg(long)]
        stderr: Option<String>,
    },
    /// Run as an MCP (Model Context Protocol) server.
    Mcp {
        #[command(subcommand)]
        subcommand: McpSubcommand,
    },
}

#[derive(Subcommand)]
pub enum McpSubcommand {
    /// Start the MCP server over stdio.
    Serve,
}

#[derive(Subcommand)]
pub enum GenerateTarget {
    Runbook,
    Changelog,
    Postmortem,
    Pr,
    All,
}

#[derive(Clone, Copy, ValueEnum)]
pub enum ExportFormat {
    Json,
    Markdown,
}
