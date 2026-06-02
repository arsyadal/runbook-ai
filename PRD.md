# RunbookAI — Agent Runbook Recorder PRD

## 1. Product Summary

RunbookAI is a local-first CLI developer tool that turns AI-assisted coding/debugging sessions into reusable engineering documentation: runbooks, changelog entries, postmortems, and next-agent briefs.

It records the useful parts of a debugging/development session—commands, exit codes, errors, changed files, technical notes, root causes, and verification steps—then generates clean Markdown artifacts that can live inside the repository.

**Core positioning:**

> RunbookAI turns AI coding agent sessions into reusable runbooks, changelogs, and postmortems.

**Core principle:**

> If an AI agent helped fix it, the process should become reusable knowledge.

## 2. Problem Statement

Developers increasingly use AI coding agents such as Claude Code, Cursor, Codex, Cline, Gemini CLI, OpenCode, pi, and Copilot to debug issues and implement changes.

However, when a session succeeds, important knowledge often remains trapped in terminal history, chat logs, or vendor-specific session state:

- commands that were run,
- errors that appeared,
- files that changed,
- failed approaches,
- root cause,
- technical decisions,
- final verification steps,
- how to fix the same issue again.

This creates several problems:

1. Debugging knowledge is not documented.
2. Developers struggle to explain how a fix was found.
3. Future agents repeat the same failed attempts.
4. Changelogs and postmortems must be written manually.
5. Team handoff is weak because session logs are too noisy.
6. AI agents solve problems but do not create durable operational memory.
7. **Token Exhaustion & Session Fragmentation:** When an agent hits its token limit or "message cap" mid-task, developers lose context. Restarting the task or switching to a different AI provider is slow and error-prone because the new agent lacks the full history of failed attempts and findings.

RunbookAI solves this by converting messy AI-assisted work into durable repo documentation that serves as a high-fidelity "save point" for any AI agent.

## 3. Differentiation

RunbookAI is **not** a replacement for AI agent session resume, chat history, or memory tools.

### 3.1 Seamless Provider Switching
RunbookAI acts as a "Universal Context Bridge." If you hit a limit in Claude Code and need to move to Cursor or Gemini, you can generate a `Next-Agent Brief` to instantly give the new provider the distilled essence of your current progress without wasting tokens on raw chat history.

### 3.2 Compared to `/resume` and session history

Tools like Claude Code, OpenCode, pi, and other coding CLIs may support session resume.

Those features are useful for continuing a session, but they are usually:

- vendor-specific,
- not designed as team documentation,
- too verbose to review,
- not PR-friendly,
- not structured as runbooks/postmortems,
- not easily reusable by another tool or human.

RunbookAI instead creates structured engineering artifacts that live in the repository.

### 3.3 Compared to memory tools such as `claude-mem`

Memory tools help agents remember facts across sessions.

RunbookAI focuses on **engineering documentation**, not agent memory alone.

The output is designed for:

- humans,
- teams,
- future AI agents,
- PR review,
- operational knowledge bases,
- incident/postmortem workflows.

**Short version:**

> Not session memory. Engineering memory.

## 4. Goals

RunbookAI aims to:

1. Record AI-assisted debugging/development sessions.
2. Capture commands, exit codes, durations, and useful output previews.
3. Detect errors from command output.
4. Detect file changes using Git.
5. Store manual notes such as decisions, findings, risks, todos, workarounds, and root causes.
6. Generate readable Markdown runbooks.
7. Generate simple changelog entries.
8. Generate simple postmortem drafts.
9. **Generate a high-fidelity Next-Agent Brief** to solve token limits and provider-switching friction.
10. Work locally without requiring an API key.

## 5. Target Users

### Primary users

- Developers using Claude Code, Cursor, Codex, Cline, Gemini CLI, OpenCode, pi, or Copilot.
- Indie hackers using AI agents daily.
- Backend/frontend/fullstack developers debugging issues with AI help.
- Open-source maintainers who want better fix documentation.

### Secondary users

- Tech leads.
- Engineering managers.
- QA engineers.
- SRE/DevOps engineers.
- Teams building operational knowledge bases.

## 6. Personas

### Persona 1 — Solo Developer

Raka uses AI agents to fix bugs in side projects. After a bug is fixed, he often forgets which commands were run and why the final solution worked.

**Goal:** Automatically produce a readable fix runbook.

### Persona 2 — Team Developer

Dina debugs backend issues with AI help. The fix process stays in a local chat/session and is hard to share with teammates.

**Goal:** Generate a runbook that can be committed or shared with the team.

### Persona 3 — Open-source Maintainer

Bayu maintains an open-source project. PRs explain what changed, but not how the issue was diagnosed.

**Goal:** Generate changelog and fix notes from actual debugging work.

## 7. MVP Scope

### 7.1 In scope for v0.1

RunbookAI v0.1 focuses on a local CLI workflow:

1. Initialize project storage.
2. Start and stop a recording session.
3. Record wrapped commands via `runbookai exec`.
4. Capture command, exit code, duration, stdout/stderr preview.
5. Detect common errors using rule-based patterns.
6. Capture Git snapshots before and after the session.
7. Detect changed files and diff stats.
8. Add manual notes.
9. Generate Markdown runbook.
10. Generate changelog draft.
11. Generate postmortem draft.
12. Generate next-agent brief section.
13. Export session data as JSON.
14. Redact common secrets from stored output.
15. Store all data locally per project.

### 7.2 Out of scope for v0.1

The following are intentionally out of scope for MVP:

1. Web dashboard.
2. Desktop GUI.
3. Cloud sync.
4. Team accounts.
5. Paid AI API summarization by default.
6. Full shell history recording.
7. VSCode/Cursor extension.
8. MCP server.
9. Remote database.
10. Automatic command execution without user action.
11. Full integration with every AI coding agent.

## 8. CLI Design

Canonical binary name for the MVP:

```bash
runbookai
```

Optional future alias:

```bash
rb
```

### 8.1 Init

```bash
runbookai init
```

Creates local config and storage.

### 8.2 Start session

```bash
runbookai start "Fix login 401 error"
```

Expected output:

```txt
RunbookAI recording started.

Session ID: rb_2026_05_29_001
Project: my-app
Branch: fix/login-error

Run commands with: runbookai exec "<command>"
Add notes with:   runbookai note --type decision "..."
Stop with:        runbookai stop
```

### 8.3 Execute and record command

```bash
runbookai exec "npm test"
runbookai exec "npm run build"
runbookai exec "docker compose up -d"
```

Captured fields:

- timestamp,
- working directory,
- command,
- exit code,
- duration,
- stdout preview,
- stderr preview,
- detected errors.

### 8.4 Add notes

```bash
runbookai note "Found issue in JWT validation."
runbookai note --type root-cause "JWT secret was missing in test environment."
runbookai note --type decision "Validate env during app bootstrap."
runbookai note --type risk "Production env variables must not be renamed."
```

Supported note types:

```txt
decision
finding
todo
risk
workaround
root-cause
```

### 8.5 Status

```bash
runbookai status
```

Displays active session summary.

### 8.6 Stop session

```bash
runbookai stop
```

Expected output:

```txt
RunbookAI recording stopped.

Session Summary:
- Duration: 34 minutes
- Commands recorded: 12
- Errors detected: 3
- Files changed: 5
- Notes added: 4

Generate documentation:
- runbookai generate runbook
- runbookai generate changelog
- runbookai generate postmortem
- runbookai generate all
```

### 8.7 Generate documentation

```bash
runbookai generate runbook
runbookai generate changelog
runbookai generate postmortem
runbookai generate all
```

### 8.8 Export

```bash
runbookai export --format json
runbookai export --format markdown
runbookai export --output ./docs/runbooks/login-fix.md
```

## 9. Functional Requirements

### 9.1 Session management

RunbookAI must:

1. Create a session ID.
2. Detect project root.
3. Capture project name.
4. Capture current Git branch if available.
5. Capture current commit hash if available.
6. Store start and end timestamps.
7. Track active/completed session status.
8. Show session status.

### 9.2 Command recording

RunbookAI must:

1. Execute commands through a wrapper.
2. Capture command string.
3. Capture exit code.
4. Capture duration.
5. Capture stdout/stderr preview.
6. Limit long output.
7. Redact secrets before storing output.
8. Detect whether command succeeded or failed.

### 9.3 Git change detection

RunbookAI must:

1. Capture Git snapshot before session.
2. Capture Git snapshot after session.
3. Detect modified, added, deleted, and renamed files.
4. Generate diff stats.
5. Store changed file list.

Internal commands may use:

```bash
git status --short
git diff --stat
git diff --name-only
```

### 9.4 Error detection

MVP uses rule-based patterns for common errors:

```txt
Error
Exception
TypeError
ReferenceError
SyntaxError
AssertionError
Compilation failed
Build failed
Test failed
Connection refused
Timeout
Permission denied
Module not found
Cannot find module
Port already in use
Migration failed
```

### 9.5 Secret redaction

RunbookAI must redact common secrets before writing logs:

```txt
API key
JWT token
password
secret
access token
private key
database URL
.env value
bearer token
```

Example:

```txt
DATABASE_URL=postgres://user:password@localhost:5432/app
```

becomes:

```txt
DATABASE_URL=[REDACTED]
```

### 9.6 Documentation generation

RunbookAI must generate:

1. `RUNBOOK.md` or `docs/runbooks/<session-slug>.md`.
2. `CHANGELOG.generated.md`.
3. `POSTMORTEM.md`.
4. JSON session export.

## 10. Non-Functional Requirements

### 10.1 Performance

1. Start session in under 1 second.
2. Generate reports in under 5 seconds for small/medium projects.
3. Add minimal overhead to wrapped commands.
4. Handle long command output with size limits.

### 10.2 Privacy

1. Local-first by default.
2. No external server by default.
3. No AI API key required for MVP.
4. Session logs stored inside project-local storage.
5. Secret redaction enabled by default.
6. User can delete sessions manually.

### 10.3 Safety

RunbookAI must not:

1. Run commands without explicit user action.
2. Modify source code without permission.
3. Commit changes automatically.
4. Upload logs to cloud.
5. Store raw secrets when redaction is enabled.

### 10.4 Compatibility

Target platforms:

```txt
macOS
Linux
Windows
```

MVP command execution model:

```txt
wrapped commands via runbookai exec
```

Shell integration is future scope.

## 11. Local Storage

Default storage:

```txt
.runbookai/
```

Structure:

```txt
.runbookai/
  config.json
  sessions/
    rb_2026_05_29_001/
      session.json
      commands.json
      notes.json
      git-before.json
      git-after.json
      output.log
```

Default generated docs:

```txt
docs/runbooks/
```

## 12. Data Model

### 12.1 Session

```ts
type Session = {
  id: string;
  title: string;
  projectName: string;
  projectPath: string;
  branchName?: string;
  commitHash?: string;
  startedAt: string;
  endedAt?: string;
  status: "active" | "completed";
  commands: CommandRecord[];
  notes: NoteRecord[];
  gitBefore?: GitSnapshot;
  gitAfter?: GitSnapshot;
};
```

### 12.2 Command record

```ts
type CommandRecord = {
  id: string;
  timestamp: string;
  cwd: string;
  command: string;
  exitCode: number;
  durationMs: number;
  stdoutPreview?: string;
  stderrPreview?: string;
  detectedErrors?: DetectedError[];
};
```

### 12.3 Note record

```ts
type NoteRecord = {
  id: string;
  timestamp: string;
  type: "decision" | "finding" | "todo" | "risk" | "workaround" | "root-cause";
  content: string;
};
```

### 12.4 Git snapshot

```ts
type GitSnapshot = {
  branch?: string;
  commitHash?: string;
  changedFiles: ChangedFile[];
};
```

### 12.5 Changed file

```ts
type ChangedFile = {
  path: string;
  status: "added" | "modified" | "deleted" | "renamed";
  additions?: number;
  deletions?: number;
};
```

### 12.6 Detected error

```ts
type DetectedError = {
  type: string;
  message: string;
  source: "stdout" | "stderr";
  severity: "low" | "medium" | "high";
};
```

## 13. Generated Output Templates

### 13.1 Runbook template

```md
# Runbook: {{title}}

## Summary

{{summary}}

## Session Info

- Session ID: {{session_id}}
- Project: {{project_name}}
- Branch: {{branch_name}}
- Started At: {{started_at}}
- Ended At: {{ended_at}}

## Problem

{{problem}}

## Root Cause

{{root_cause}}

## Commands Run

{{commands_table}}

## Errors Encountered

{{errors}}

## Files Changed

{{changed_files}}

## Decisions

{{decisions}}

## Fix Applied

{{fix_applied}}

## Verification

{{verification_steps}}

## How to Fix This Again

{{how_to_fix_again}}

## Next-Agent Brief

{{next_agent_brief}}

## Risks and Notes

{{risks}}
```

### 13.2 Changelog template

```md
# Changelog Entry

## Summary

{{summary}}

## Fixed

{{fixed_items}}

## Changed

{{changed_items}}

## Files Changed

{{changed_files}}

## Verification

{{verification_commands}}
```

### 13.3 Postmortem template

```md
# Postmortem: {{title}}

## Incident Summary

{{incident_summary}}

## Impact

{{impact}}

## Root Cause

{{root_cause}}

## Timeline

{{timeline}}

## Resolution

{{resolution}}

## What Went Well

{{went_well}}

## What Could Be Improved

{{could_be_improved}}

## Action Items

{{action_items}}
```

## 14. MVP Acceptance Criteria

RunbookAI v0.1 is complete when a user can run:

```bash
runbookai init
runbookai start "Fix login 401 error"
runbookai exec "npm test"
runbookai note --type root-cause "JWT secret was missing in test environment."
runbookai exec "npm run build"
runbookai stop
runbookai generate runbook
```

And receive a Markdown runbook containing:

1. Session summary.
2. Problem description.
3. Command history.
4. Detected errors.
5. Changed files.
6. Technical decisions.
7. Root cause.
8. Fix summary.
9. Verification steps.
10. How to fix this again.
11. Next-agent brief.

Additional v0.1 success criteria:

1. Works without API keys.
2. Uses local storage only.
3. Redacts common secrets.
4. Records at least one end-to-end session successfully.
5. Runs on at least macOS and Linux initially; Windows support should be planned/tested.
6. Can be used on at least three different repositories during dogfooding.

## 15. Recommended Tech Stack

Recommended implementation:

```txt
Language: Rust
CLI: clap
Serialization: serde, serde_json
Terminal output: owo-colors or similar
Git integration: shell git commands first, git2 later if needed
Markdown generation: templates or custom renderer
Testing: assert_cmd, insta
Distribution: cargo install, GitHub Releases, Homebrew later
```

Reasoning:

1. CLI-first product.
2. Local-first workflow.
3. Single binary distribution.
4. Strong fit for filesystem, process, and Git operations.
5. Good positioning as a serious developer tool.

## 16. Roadmap

### Phase 1 — MVP CLI

Target: 2–4 weeks.

Features:

1. `init`.
2. `start`, `status`, `stop`.
3. Wrapped command recording.
4. Git diff detection.
5. Manual notes.
6. Error detection.
7. Secret redaction.
8. Runbook generation.
9. Changelog generation.
10. Postmortem generation.
11. JSON export.

### Phase 2 — Better Developer Experience

Target: 1–2 months after MVP.

Features:

1. Shell integration.
2. Custom templates.
3. Better Markdown output.
4. Search old runbooks.
5. Better secret redaction.
6. Better Git diff summary.
7. Short alias `rb`.

### Phase 3 — AI and Agent Integration

Target: after initial traction.

Features:

1. AI-assisted summaries.
2. Local LLM support.
3. MCP server.
4. Agent auto-note integration.
5. PR description generator.
6. Next-agent context pack.

### Phase 4 — Team/Pro Mode

Only after traction.

Features:

1. Shared runbook dashboard.
2. GitHub App.
3. Team templates.
4. Multi-repo search.
5. Incident management workflows.

## 17. Future Features

### 17.1 AI-assisted summary

```bash
runbookai generate runbook --ai
```

Potential providers:

```txt
Anthropic
OpenAI
Gemini
Ollama
LM Studio
```

Default MVP must not require these providers.

### 17.2 MCP server

```bash
runbookai mcp serve
```

Use cases:

- AI agent queries previous runbooks.
- AI agent saves new findings.
- AI agent searches similar issues.

### 17.3 Searchable knowledge base

```bash
runbookai search "login 401"
runbookai search "database timeout"
runbookai search "docker port already in use"
```

### 17.4 GitHub PR integration

```bash
runbookai generate pr
```

Output:

```md
## Summary

Fixed login 401 issue caused by missing JWT secret in test environment.

## Changes

- Added env validation.
- Updated login test config.
- Improved error handling.

## Verification

- npm test
- npm run build

## Runbook

See: docs/runbooks/login-401-fix.md
```

## 18. Relationship With ContextLint

RunbookAI can later become part of a broader ContextOps toolkit.

Lifecycle:

```txt
ContextLint  = before AI session: audit and clean context
RunbookAI    = during/after AI session: preserve process as reusable knowledge
```

Workflow:

```txt
contextlint scan
↓
AI coding agent session
↓
runbookai record/generate
↓
runbook becomes future context
↓
contextlint prevents memory/docs from becoming bloated
```

Do not merge the tools during MVP. Keep RunbookAI focused on session-to-runbook conversion.

## 19. Metrics

### Product metrics

1. Number of sessions recorded.
2. Number of runbooks generated.
3. Number of changelogs generated.
4. Number of postmortems generated.
5. Average commands per session.
6. Average files changed per session.
7. Percentage of sessions with notes.

### Open-source metrics

1. GitHub stars.
2. Issues opened.
3. Pull requests.
4. External feedback.
5. Mentions by AI coding users.

### First 30-day targets

Realistic:

```txt
GitHub stars: 100+
External feedback: 10+
Useful issues: 5+
Contributors: 1–2
```

Ambitious:

```txt
GitHub stars: 500+
Used in real public repos
Mentioned by AI coding community
```

## 20. Risks and Mitigations

### 20.1 Privacy risk

Command output may contain secrets.

Mitigation:

1. Secret redaction enabled by default.
2. Output preview limits.
3. No cloud upload by default.
4. Warning before export.

### 20.2 Noisy output

Long command output can make reports unreadable.

Mitigation:

1. Limit previews.
2. Store full output separately if needed.
3. Extract errors separately.
4. Keep generated runbook concise.

### 20.3 User workflow friction

Users may not want to run commands through `runbookai exec`.

Mitigation:

1. Start with wrapped commands for MVP.
2. Add shell integration later.
3. Provide alias support.
4. Keep commands simple.

### 20.4 Generic generated docs

Reports may feel too template-like.

Mitigation:

1. Encourage notes.
2. Use actual command/error/Git data.
3. Add next-agent brief.
4. Add AI-assisted summary later.

## 21. Development Breakdown

### Week 1

1. Set up Rust CLI project.
2. Implement `runbookai init`.
3. Implement `.runbookai` storage.
4. Implement session ID generation.
5. Implement `start`, `status`, and `stop`.

### Week 2

1. Implement `runbookai exec`.
2. Capture command, exit code, duration.
3. Capture stdout/stderr preview.
4. Implement basic error detection.
5. Implement secret redaction.

### Week 3

1. Implement Git snapshot before/after.
2. Detect changed files.
3. Implement `runbookai note`.
4. Store notes in session.
5. Add JSON export.

### Week 4

1. Implement Markdown generator.
2. Generate runbook.
3. Generate changelog.
4. Generate postmortem.
5. Add tests.
6. Update README.
7. Prepare demo GIF/video.
8. Publish first release.

## 22. Example README Headline

```md
# RunbookAI

Turn your AI coding agent sessions into reusable runbooks, changelogs, and postmortems.

RunbookAI records commands, changed files, errors, and technical decisions during debugging, then generates clean documentation so your fix process never disappears in terminal history again.
```

## 23. Final MVP Definition

RunbookAI v0.1 is a **session-to-runbook compiler** for AI-assisted development.

It is done when a developer can record a real debugging session from start to finish and generate a useful Markdown runbook that explains:

1. what problem happened,
2. what commands were run,
3. what errors were found,
4. what files changed,
5. what decisions were made,
6. what fixed the issue,
7. how the fix was verified,
8. how to fix it again next time.
