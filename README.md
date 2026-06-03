# RunbookAI

Turn your AI coding agent sessions into reusable runbooks, changelogs, and postmortems.

> **Don't worry about your AI agent's limit. RunbookAI has your back.**

RunbookAI is your **Context Insurance**. It records commands, changed files, errors, and technical decisions during debugging, then generates clean documentation so your progress never vanishes—even if your AI agent hits a token limit or you need to switch providers mid-session.

## The "Dreaded" Session Reset is Over

Ever hit a "message cap" mid-debugging? Switching from Claude to Cursor and having to explain everything again? 

RunbookAI acts as your **Universal Context Bridge**. Generate a `Next-Agent Brief` and give it to your next AI provider. They'll pick up exactly where you left off, saving you time, tokens, and sanity.

> Not session memory. Engineering memory.

## Why

AI coding tools like Claude Code, Cursor, Codex, Gemini CLI, OpenCode, pi, and Copilot can help fix bugs fast. But the useful process often stays trapped in chat/session logs:

- what commands were run,
- what errors appeared,
- what files changed,
- what failed,
- what root cause was found,
- how the fix was verified.

RunbookAI converts that process into repo-friendly Markdown documentation.

## Status

Early Rust MVP — modular, tested, and lint-clean.

Implemented:

- Session lifecycle: `init`, `start`, `status`, `stop`
- Environment diagnostics: `doctor`
- Command capture: `exec "<command>"` and shell-hook based recording
- Manual notes: `note --type <kind> "..."`
- Documentation generation: `generate runbook`, `changelog`, `postmortem`, `pr`, `all`
- Session export: `export --format json|markdown`
- Session search: `search "<query>"`
- Shell helpers: `alias`, `shell-hook`
- MCP server: `mcp serve`
- Custom Handlebars templates (`.runbookai/templates/`)
- Enhanced Git diff capture with redaction
- Local `.runbookai/` storage
- Git changed-file detection
- Basic error detection
- Basic secret redaction
- 43 tests (29 unit + 14 integration)
- Zero clippy warnings

## Shell Integration (No More `exec`)

If you don't want to type `runbookai exec` for every command, you can integrate RunbookAI directly into your shell. This will automatically record every command you run whenever a session is active.

Add this to your `.zshrc` or `.bashrc`:

```bash
# For Zsh
source <(runbookai shell-hook zsh)

# For Bash
source <(runbookai shell-hook bash)
```

Now, simply running `runbookai start "title"` is enough. Every subsequent command in that shell will be recorded automatically until you run `runbookai stop`.

*Note: Shell integration captures command strings, exit codes, and duration. For full output and error capture, `runbookai exec` is still recommended.*

## MCP Server Support

RunbookAI supports the **Model Context Protocol (MCP)**. This allows AI agents (like Claude Desktop or Cursor) to directly query your previous sessions.

To use it with Claude Desktop, add this to your `claude_desktop_config.json`:

```json
{
  "mcpServers": {
    "runbookai": {
      "command": "runbookai",
      "args": ["mcp", "serve"]
    }
  }
}
```

Capabilities:
- **Tools:** `search_sessions` (search through old sessions).
- **Resources:** `runbook://sessions/<id>` (read full session data as JSON).

## Install from source

```bash
cargo install --path .
```

Or run directly:

```bash
cargo run -- --help
```

## Windows build notes

On Windows, use one of these setups before running `cargo test` or `cargo clippy`:

- MSVC: install **Visual Studio Build Tools** with the **Desktop development with C++** workload, then run Cargo from a Developer PowerShell/Command Prompt.
- GNU: install MinGW binutils so `dlltool.exe` is available on `PATH`.

If Git Bash resolves `link.exe` to `C:\Program Files\Git\usr\bin\link.exe`, the MSVC linker is not being used.

## Quickstart

Initialize storage:

```bash
runbookai init
```

Start a session:

```bash
runbookai start "Fix login 401 error"
```

Run commands through RunbookAI:

```bash
runbookai exec "npm test"
runbookai exec "npm run build"
```

Add notes:

```bash
runbookai note --type root-cause "JWT secret was missing in test environment."
runbookai note --type decision "Validate env during app bootstrap."
runbookai note --type risk "Production env variables must not be renamed."
```

Check status:

```bash
runbookai status
```

Diagnose your local setup:

```bash
runbookai doctor
```

Stop recording:

```bash
runbookai stop
```

Generate docs:

```bash
runbookai generate all
```

Generated files are written to:

```txt
docs/runbooks/
```

Session data is stored locally in:

```txt
.runbookai/
```

## Project Structure

```
src/
  main.rs      # CLI entry point and command routing
  cli.rs       # Clap argument definitions
  models.rs    # Domain types (Session, CommandRecord, Note, etc.)
  config.rs    # Config loading
  doctor.rs    # Environment diagnostics
  session.rs   # Session lifecycle (init, start, stop, status, load, save)
  git.rs       # Git snapshot and diff parsing
  command.rs   # Command execution and note recording
  redact.rs    # Secret redaction
  detect.rs    # Error pattern detection
  render.rs    # Markdown generators (runbook, changelog, postmortem)
  export.rs    # JSON / Markdown export
  util.rs      # Shared helpers
```

## Example workflow

```bash
runbookai init
runbookai start "Fix auth test failure"
runbookai exec "npm test"
runbookai note --type finding "Login test fails with 401 for valid credentials."
runbookai note --type root-cause "JWT secret is missing in the test environment."
runbookai exec "npm run build"
runbookai stop
runbookai generate runbook
```

Example output:

```txt
docs/runbooks/2026-06-01-fix-auth-test-failure.md
```

The generated runbook includes:

- session summary,
- command history,
- detected errors,
- changed files,
- decisions,
- root cause,
- verification steps,
- failed attempts,
- next-agent brief.

## How it differs from `/resume` and memory tools

`/resume` and session history help continue a conversation inside one AI tool.

RunbookAI creates durable engineering artifacts that live in your repo and can be read by humans, teammates, and future agents.

## Relationship with ContextLint

- ContextLint = before AI session: audit and clean project context.
- RunbookAI = during/after AI session: preserve the fix process as reusable knowledge.

They are separate tools but can become part of a broader ContextOps workflow later.

## Product principle

If an AI agent helped fix it, the process should become reusable knowledge.
