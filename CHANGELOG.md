# Changelog

All notable changes in RunbookAI, from the first commit to the latest update.

## Unreleased

### Documentation
- Added real-world use cases to the README to make RunbookAI's practical value clearer.
- Covered production incidents, AI-agent handoff, provider switching, PR descriptions, team knowledge, and risky-change audits.

## 2026-06-05

### `9d9d9bf` — docs: add visual comparison assets
- Added visual comparison assets for RunbookAI:
  - `docs/assets/runbook-comparison.svg`
  - `docs/assets/runbook-comparison.png`
  - `docs/assets/runbook-comparison.jpg`
  - `docs/assets/runbook-comparison.mp4`
- Displayed the comparison infographic in `README.md`.
- Added README links to the PNG, JPG, and MP4 assets.

### `2d52503` — docs: add runbook comparison
- Added a README comparison table showing the difference between using RunbookAI and working without RunbookAI.
- Covered session continuity, command history, error tracking, changed files, root cause capture, AI-agent handoff, team documentation, and token savings.

## 2026-06-03

### `695a508` — ci: publish release assets with gh
- Updated the release workflow to publish build assets with GitHub CLI (`gh`).
- This commit is tagged as the `v0.1.0` release.

### `813c4be` — ci: attach release archives
- Updated the release workflow so build archives are attached to GitHub Releases.

### `091e58b` — ci: fix Rust toolchain action
- Fixed the Rust toolchain configuration in CI and release workflows.

### `82a6081` — docs: add v0.1.0 release notes
- Added release notes for `v0.1.0` in `docs/releases/v0.1.0.md`.
- Updated release workflow behavior around the release process.

### `463d6f2` — ci: add release workflow
- Added a GitHub Actions workflow for automated release builds.
- Documented the tagged release process in the README.

### `f4d2db3` — feat: improve diagnostics and shell support
- Improved local environment diagnostics.
- Enhanced shell support.
- Updated the README, CLI, doctor module, and integration tests.

### `0c8a100` — feat: add doctor command
- Added the `runbookai doctor` command for local environment diagnostics.
- Added JSON output support for the doctor command.
- Added integration tests for doctor functionality.

### `8aee713` — chore: clean docs and formatting
- Cleaned up documentation and code formatting.
- Refined several modules and integration tests.

## 2026-06-02

### `ba597e4` — feat: implement GitHub PR Description Generator (Phase 4)
- Added the `runbookai generate pr` command.
- Added a default PR description template with Summary, Changes, and Verification sections.
- Integrated session notes such as Decisions and Risks into the PR template.
- Updated the README and integration tests.

### `dd8eae7` — feat: implement Full Shell Integration (No-Wrapper Recording)
- Added the `shell-hook` command to generate Zsh/Bash integration scripts.
- Added the `record` command for headless command-result recording.
- Implemented `record_headless` logic for background logging.
- Updated the README with shell integration instructions.

### `a0635ee` — feat: implement MCP Server support (Phase 3)
- Added an MCP module implementing JSON-RPC over stdio.
- Added the `runbookai mcp serve` command.
- Exposed `search_sessions` as an MCP tool.
- Exposed session data as MCP resources using `runbook://sessions/<id>`.
- Added a Claude Desktop configuration example to the README.

### `ad654ce` — marketing: reposition as Context Insurance for AI token limits
- Repositioned RunbookAI as “Context Insurance”.
- Strengthened the “Universal Context Bridge” message for switching AI providers.
- Updated the PRD, README, and Next-Agent Brief template.
- Clarified messaging around context loss when an AI agent reaches its limit.

### `7643609` — docs: update PRD to address token limits and context loss
- Added problem-statement coverage for token exhaustion and session fragmentation.
- Described RunbookAI as a Universal Context Bridge.
- Added the high-fidelity Next-Agent Brief as a core product goal.

### `dc61743` — feat: implement AI-Assisted Summaries (Phase 3)
- Added the `ai` module with support for Ollama, OpenAI, and Gemini.
- Added the `--ai` flag to the `generate` command.
- Integrated AI summaries into runbook rendering.
- Converted the entry point to async with Tokio.
- Added `reqwest` and `tokio` dependencies.

### `83b1a69` — feat: implement phase 2 core features
- Added custom template support powered by Handlebars.
- Added enhanced Git diff capture with redaction.
- Added the `search` command for querying previous sessions.
- Added the `alias` command for shell integration.
- Updated the data model to store session diffs.
- Confirmed all tests were passing at the time.

### `2860ae8` — test: add integration tests and CI workflow
- Added integration tests for secret redaction and export formats.
- Added testing dev dependencies such as `assert_cmd` and `predicates`.
- Added GitHub Actions CI for multi-platform testing.

### `2efc231` — refactor: split monolithic main.rs into modular architecture
- Split the monolithic `main.rs` into a modular architecture.
- Added core modules for CLI, command handling, config, detection, export, Git integration, models, redaction, rendering, session management, and utilities.
- Added 29 unit tests for redaction, detection, utilities, and rendering.
- Fixed a Clippy warning by extracting a complex type into `DiffStats`.
- Updated the README with project structure and project status.

## 2026-05-29

### `df2af10` — first commit
- Initial project commit.
- Added the first README as the documentation foundation for RunbookAI.
