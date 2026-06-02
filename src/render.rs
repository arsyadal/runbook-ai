use crate::models::{ChangedFile, NoteKind, Session};
use crate::util::{escape_table, local_time, markdown_list_or_placeholder, non_empty};
use slug::slugify;

pub fn render_runbook(session: &Session) -> String {
    let root_cause = notes_by_kind(session, NoteKind::RootCause);
    let decisions = notes_by_kind(session, NoteKind::Decision);
    let risks = notes_by_kind(session, NoteKind::Risk);
    let verification = successful_commands(session);
    let failed = failed_commands(session);

    format!(
        "# Runbook: {title}\n\n\
        ## Summary\n\n\
        This runbook was generated from a RunbookAI session. It captures the commands, errors, changed files, notes, and verification steps from the debugging/development process.\n\n\
        ## Session Info\n\n\
        - Session ID: `{id}`\n\
        - Project: `{project}`\n\
        - Branch: {branch}\n\
        - Started At: {started}\n\
        - Ended At: {ended}\n\n\
        ## Problem\n\n\
        {problem}\n\n\
        ## Root Cause\n\n\
        {root_cause}\n\n\
        ## Commands Run\n\n\
        {commands}\n\n\
        ## Errors Encountered\n\n\
        {errors}\n\n\
        ## Files Changed\n\n\
        {files}\n\n\
        ## Decisions\n\n\
        {decisions}\n\n\
        ## Fix Applied\n\n\
        Review the changed files and successful verification commands above. Add a `decision` or `root-cause` note during the session to make this section more specific.\n\n\
        ## Verification\n\n\
        {verification}\n\n\
        ## Failed Attempts\n\n\
        {failed}\n\n\
        ## How to Fix This Again\n\n\
        1. Review the root cause and decisions in this runbook.\n\
        2. Inspect the changed files listed above.\n\
        3. Re-run the verification commands.\n\
        4. Check the errors encountered section if the issue reappears.\n\n\
        ## Next-Agent Brief\n\n\
        When continuing this work, read the root cause, files changed, failed attempts, and verification sections first. Avoid repeating failed commands unless the environment has changed.\n\n\
        ## Risks and Notes\n\n\
        {risks}\n",
        title = session.title,
        id = session.id,
        project = session.project_name,
        branch = session.branch_name.as_deref().unwrap_or("unknown"),
        started = local_time(session.started_at),
        ended = session.ended_at.map(local_time).unwrap_or_else(|| "active".to_string()),
        problem = non_empty(&session.title, "No problem description recorded."),
        root_cause = markdown_list_or_placeholder(root_cause, "No root cause note recorded yet."),
        commands = render_commands_table(session),
        errors = render_errors(session),
        files = render_changed_files(session),
        decisions = markdown_list_or_placeholder(decisions, "No decision notes recorded."),
        verification = markdown_list_or_placeholder(verification, "No successful verification commands recorded."),
        failed = markdown_list_or_placeholder(failed, "No failed commands recorded."),
        risks = markdown_list_or_placeholder(risks, "No risks recorded."),
    )
}

pub fn render_changelog(session: &Session) -> String {
    format!(
        "# Changelog Entry: {title}\n\n\
        ## Summary\n\n\
        - {title}\n\n\
        ## Changed Files\n\n\
        {files}\n\n\
        ## Verification\n\n\
        {verification}\n",
        title = session.title,
        files = render_changed_files(session),
        verification = markdown_list_or_placeholder(
            successful_commands(session),
            "No successful verification commands recorded."
        ),
    )
}

pub fn render_postmortem(session: &Session) -> String {
    format!(
        "# Postmortem: {title}\n\n\
        ## Incident Summary\n\n\
        {title}\n\n\
        ## Impact\n\n\
        Not recorded. Add a `risk` or `finding` note to capture impact.\n\n\
        ## Root Cause\n\n\
        {root_cause}\n\n\
        ## Timeline\n\n\
        {timeline}\n\n\
        ## Resolution\n\n\
        Review successful commands and changed files.\n\n\
        ## What Went Well\n\n\
        - Session data was recorded.\n\n\
        ## What Could Be Improved\n\n\
        - Add more notes during the session for richer postmortems.\n\n\
        ## Action Items\n\n\
        {todos}\n",
        title = session.title,
        root_cause = markdown_list_or_placeholder(
            notes_by_kind(session, NoteKind::RootCause),
            "No root cause note recorded."
        ),
        timeline = render_timeline(session),
        todos = markdown_list_or_placeholder(
            notes_by_kind(session, NoteKind::Todo),
            "No action items recorded."
        ),
    )
}

fn render_commands_table(session: &Session) -> String {
    if session.commands.is_empty() {
        return "No commands recorded.".to_string();
    }
    let mut out = String::from("| Time | Command | Exit Code | Duration |\n|---|---|---:|---:|\n");
    for command in &session.commands {
        out.push_str(&format!(
            "| {} | `{}` | {} | {}ms |\n",
            local_time(command.timestamp),
            escape_table(&command.command),
            command.exit_code,
            command.duration_ms
        ));
    }
    out
}

fn render_errors(session: &Session) -> String {
    let mut lines = Vec::new();
    for command in &session.commands {
        for error in &command.detected_errors {
            lines.push(format!(
                "- `{}` → **{}** from {}: {}",
                command.command, error.kind, error.source, error.message
            ));
        }
    }
    markdown_list_or_placeholder(lines, "No errors detected.")
}

fn render_changed_files(session: &Session) -> String {
    let files = changed_files(session)
        .iter()
        .map(|file| {
            let stats = match (file.additions, file.deletions) {
                (Some(add), Some(del)) => format!(" (+{add}/-{del})"),
                _ => String::new(),
            };
            format!("- `{}` — {}{}", file.path, file.status, stats)
        })
        .collect::<Vec<_>>();
    markdown_list_or_placeholder(files, "No changed files detected.")
}

fn render_timeline(session: &Session) -> String {
    let mut rows = String::from("| Time | Event |\n|---|---|\n");
    rows.push_str(&format!(
        "| {} | Session started |\n",
        local_time(session.started_at)
    ));
    for command in &session.commands {
        rows.push_str(&format!(
            "| {} | Ran `{}` (exit {}) |\n",
            local_time(command.timestamp),
            escape_table(&command.command),
            command.exit_code
        ));
    }
    for note in &session.notes {
        rows.push_str(&format!(
            "| {} | Added {} note: {} |\n",
            local_time(note.timestamp),
            note.kind,
            escape_table(&note.content)
        ));
    }
    if let Some(ended_at) = session.ended_at {
        rows.push_str(&format!("| {} | Session stopped |\n", local_time(ended_at)));
    }
    rows
}

pub fn changed_files(session: &Session) -> Vec<&ChangedFile> {
    session
        .git_after
        .as_ref()
        .or(session.git_before.as_ref())
        .map(|snapshot| snapshot.changed_files.iter().collect())
        .unwrap_or_default()
}

fn notes_by_kind(session: &Session, kind: NoteKind) -> Vec<String> {
    session
        .notes
        .iter()
        .filter(|note| std::mem::discriminant(&note.kind) == std::mem::discriminant(&kind))
        .map(|note| note.content.clone())
        .collect()
}

fn successful_commands(session: &Session) -> Vec<String> {
    session
        .commands
        .iter()
        .filter(|command| command.exit_code == 0)
        .map(|command| format!("`{}`", command.command))
        .collect()
}

fn failed_commands(session: &Session) -> Vec<String> {
    session
        .commands
        .iter()
        .filter(|command| command.exit_code != 0)
        .map(|command| format!("`{}` exited with {}", command.command, command.exit_code))
        .collect()
}

pub fn session_slug(session: &Session) -> String {
    format!(
        "{}-{}",
        session.started_at.format("%Y-%m-%d"),
        slugify(&session.title)
    )
}

pub fn count_errors(session: &Session) -> usize {
    session
        .commands
        .iter()
        .map(|command| command.detected_errors.len())
        .sum()
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::models::{
        ChangedFile, CommandRecord, DetectedError, GitSnapshot, NoteRecord, Session, SessionStatus,
    };
    use chrono::Utc;

    fn sample_session() -> Session {
        Session {
            id: "rb_2026_01_01_000000_12345678".to_string(),
            title: "Fix login error".to_string(),
            project_name: "my-app".to_string(),
            project_path: "/home/user/my-app".to_string(),
            branch_name: Some("main".to_string()),
            commit_hash: Some("abc123".to_string()),
            started_at: Utc::now(),
            ended_at: Some(Utc::now()),
            status: SessionStatus::Completed,
            commands: vec![
                CommandRecord {
                    id: "cmd-1".to_string(),
                    timestamp: Utc::now(),
                    cwd: "/home/user/my-app".to_string(),
                    command: "npm test".to_string(),
                    exit_code: 1,
                    duration_ms: 1200,
                    stdout_preview: Some("Test failed".to_string()),
                    stderr_preview: None,
                    detected_errors: vec![DetectedError {
                        kind: "Test failed".to_string(),
                        message: "Test failed: login".to_string(),
                        source: "stdout".to_string(),
                        severity: "high".to_string(),
                    }],
                },
                CommandRecord {
                    id: "cmd-2".to_string(),
                    timestamp: Utc::now(),
                    cwd: "/home/user/my-app".to_string(),
                    command: "npm run build".to_string(),
                    exit_code: 0,
                    duration_ms: 3400,
                    stdout_preview: Some("Build successful".to_string()),
                    stderr_preview: None,
                    detected_errors: vec![],
                },
            ],
            notes: vec![
                NoteRecord {
                    id: "note-1".to_string(),
                    timestamp: Utc::now(),
                    kind: NoteKind::RootCause,
                    content: "JWT secret was missing".to_string(),
                },
                NoteRecord {
                    id: "note-2".to_string(),
                    timestamp: Utc::now(),
                    kind: NoteKind::Decision,
                    content: "Add env validation".to_string(),
                },
            ],
            git_before: None,
            git_after: Some(GitSnapshot {
                branch: Some("main".to_string()),
                commit_hash: Some("abc123".to_string()),
                changed_files: vec![
                    ChangedFile {
                        path: "src/auth.rs".to_string(),
                        status: "modified".to_string(),
                        additions: Some(10),
                        deletions: Some(2),
                    },
                    ChangedFile {
                        path: "tests/login.rs".to_string(),
                        status: "added".to_string(),
                        additions: Some(45),
                        deletions: None,
                    },
                ],
            }),
        }
    }

    #[test]
    fn session_slug_format() {
        let session = sample_session();
        let slug = session_slug(&session);
        let expected_prefix = format!("{}-fix-login-error", session.started_at.format("%Y-%m-%d"));
        assert!(slug.starts_with(&expected_prefix));
    }

    #[test]
    fn count_errors_sums_all() {
        let session = sample_session();
        assert_eq!(count_errors(&session), 1);
    }

    #[test]
    fn render_runbook_contains_title() {
        let session = sample_session();
        let output = render_runbook(&session);
        assert!(output.contains("# Runbook: Fix login error"));
    }

    #[test]
    fn render_runbook_contains_root_cause() {
        let session = sample_session();
        let output = render_runbook(&session);
        assert!(output.contains("JWT secret was missing"));
    }

    #[test]
    fn render_runbook_contains_commands_table() {
        let session = sample_session();
        let output = render_runbook(&session);
        assert!(output.contains("npm test"));
        assert!(output.contains("npm run build"));
    }

    #[test]
    fn render_changelog_contains_title() {
        let session = sample_session();
        let output = render_changelog(&session);
        assert!(output.contains("# Changelog Entry: Fix login error"));
    }

    #[test]
    fn render_postmortem_contains_timeline() {
        let session = sample_session();
        let output = render_postmortem(&session);
        assert!(output.contains("Session started"));
        assert!(output.contains("npm test"));
    }

    #[test]
    fn changed_files_prefers_git_after() {
        let session = sample_session();
        let files = changed_files(&session);
        assert_eq!(files.len(), 2);
        assert_eq!(files[0].path, "src/auth.rs");
    }

    #[test]
    fn successful_commands_filters_zero_exit() {
        let session = sample_session();
        let cmds = successful_commands(&session);
        assert_eq!(cmds.len(), 1);
        assert!(cmds[0].contains("npm run build"));
    }

    #[test]
    fn failed_commands_filters_nonzero_exit() {
        let session = sample_session();
        let cmds = failed_commands(&session);
        assert_eq!(cmds.len(), 1);
        assert!(cmds[0].contains("npm test"));
    }
}
