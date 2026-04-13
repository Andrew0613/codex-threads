use std::collections::{HashMap, HashSet, VecDeque};
use std::fmt::Write as _;
use std::fs::{self, File};
use std::io::{BufRead, BufReader};
use std::path::{Path, PathBuf};
use std::time::UNIX_EPOCH;

use anyhow::{Context, Result, anyhow, bail};
use clap::{Args, Parser, Subcommand};
use rusqlite::{Connection, OptionalExtension, Transaction, params};
use serde::Serialize;
use serde_json::{Value, json};
use walkdir::WalkDir;

const PARSER_VERSION: i64 = 2;

fn main() {
    let cli = Cli::parse();
    let json_mode = cli.json;

    match run(cli) {
        Ok(output) => {
            if json_mode {
                println!(
                    "{}",
                    serde_json::to_string_pretty(&output.json).expect("json serialization failed")
                );
            } else {
                println!("{}", output.text);
            }
        }
        Err(error) => {
            if json_mode {
                let payload = json!({
                    "ok": false,
                    "error": {
                        "message": error.to_string(),
                    }
                });
                eprintln!(
                    "{}",
                    serde_json::to_string_pretty(&payload).expect("json serialization failed")
                );
            } else {
                eprintln!("error: {error:#}");
            }
            std::process::exit(1);
        }
    }
}

#[derive(Debug, Parser)]
#[command(name = "codex-threads")]
#[command(about = "Search and read local Codex session archives", long_about = None)]
struct Cli {
    #[arg(long, global = true)]
    json: bool,

    #[arg(long, global = true)]
    db: Option<PathBuf>,

    #[arg(long, global = true)]
    sessions: Option<PathBuf>,

    #[command(subcommand)]
    command: Command,
}

#[derive(Debug, Subcommand)]
enum Command {
    Sync,
    Doctor,
    Insights(InsightsArgs),
    Messages {
        #[command(subcommand)]
        command: MessagesCommand,
    },
    Threads {
        #[command(subcommand)]
        command: ThreadsCommand,
    },
    Events {
        #[command(subcommand)]
        command: EventsCommand,
    },
}

#[derive(Debug, Subcommand)]
enum MessagesCommand {
    Search(SearchArgs),
}

#[derive(Debug, Subcommand)]
enum ThreadsCommand {
    Resolve(SearchArgs),
    Recent(RecentThreadsArgs),
    Read(ReadThreadArgs),
    Summarize(SummarizeThreadArgs),
    Insight(SummarizeThreadArgs),
}

#[derive(Debug, Subcommand)]
enum EventsCommand {
    Read(ReadEventsArgs),
    Summary(ReadEventsArgs),
}

#[derive(Debug, Args)]
struct SearchArgs {
    query: String,

    #[arg(long, default_value_t = 20)]
    limit: usize,
}

#[derive(Debug, Args)]
struct ReadThreadArgs {
    session_id: String,

    #[arg(long)]
    limit: Option<usize>,

    #[arg(long, default_value_t = 1600)]
    max_message_chars: usize,

    #[arg(long)]
    raw: bool,
}

#[derive(Debug, Args)]
struct SummarizeThreadArgs {
    session_id: String,

    #[arg(long, default_value_t = 120)]
    event_limit: usize,

    #[arg(long, default_value_t = 1600)]
    max_message_chars: usize,
}

#[derive(Debug, Args)]
struct InsightsArgs {
    #[arg(long, default_value_t = 50)]
    limit: usize,

    #[arg(long)]
    project: Option<String>,

    #[arg(long)]
    output: Option<PathBuf>,

    #[arg(long, default_value_t = 120)]
    event_limit: usize,

    #[arg(long, default_value_t = 1600)]
    max_message_chars: usize,
}

#[derive(Debug, Args)]
struct RecentThreadsArgs {
    #[arg(long, default_value_t = 20)]
    limit: usize,

    #[arg(long)]
    cwd: Option<String>,

    #[arg(long)]
    project: Option<String>,
}

#[derive(Debug, Args)]
struct ReadEventsArgs {
    session_id: String,

    #[arg(long, default_value_t = 50)]
    limit: usize,
}

struct RenderedOutput {
    text: String,
    json: Value,
}

#[derive(Debug, Clone, Serialize)]
struct ThreadSummary {
    thread_id: String,
    path: String,
    started_at: Option<String>,
    cwd: Option<String>,
    project_name: Option<String>,
    summary: Option<String>,
    originator: Option<String>,
    cli_version: Option<String>,
    model_provider: Option<String>,
    agent_nickname: Option<String>,
    agent_role: Option<String>,
    message_count: usize,
    event_count: usize,
}

#[derive(Debug, Clone, Serialize)]
struct SearchMessageResult {
    thread_id: String,
    timestamp: Option<String>,
    role: String,
    phase: Option<String>,
    summary: Option<String>,
    snippet: String,
    cwd: Option<String>,
    path: String,
}

#[derive(Debug, Clone, Serialize)]
struct ThreadResolveResult {
    thread_id: String,
    started_at: Option<String>,
    cwd: Option<String>,
    project_name: Option<String>,
    summary: Option<String>,
    path: String,
    message_count: usize,
    event_count: usize,
    match_count: usize,
    last_match_at: Option<String>,
    sort_reason: String,
}

#[derive(Debug, Clone, Serialize)]
struct ThreadMessage {
    timestamp: Option<String>,
    role: String,
    phase: Option<String>,
    text: String,
    cleaned: bool,
    original_chars: Option<usize>,
}

#[derive(Debug, Clone, Serialize)]
struct RecentThreadResult {
    thread_id: String,
    started_at: Option<String>,
    cwd: Option<String>,
    project_name: Option<String>,
    summary: Option<String>,
    path: String,
    message_count: usize,
    event_count: usize,
}

#[derive(Debug, Clone, Serialize)]
struct ThreadDigest {
    thread: ThreadSummary,
    objective: Option<String>,
    key_requests: Vec<String>,
    key_actions: Vec<EventSummaryItem>,
    outcome: Option<String>,
}

#[derive(Debug, Clone, Serialize)]
struct ThreadInsight {
    thread: ThreadSummary,
    title: String,
    summary: String,
    insights: Vec<String>,
    evidence: Vec<String>,
    next_steps: Vec<String>,
    content: String,
}

include!("insights.rs");

fn summarize_event_stream(events: &[EventItem]) -> Vec<EventSummaryItem> {
    let mut summary = Vec::new();
    let mut suppressed = 0usize;
    let mut suppressed_timestamp = None;

    for event in events {
        if let Some(item) = summarize_event_item(event) {
            if suppressed > 0 {
                summary.push(EventSummaryItem {
                    timestamp: suppressed_timestamp.take(),
                    category: "suppressed".to_owned(),
                    detail: format!("suppressed {suppressed} low-signal events"),
                });
                suppressed = 0;
            }
            summary.push(item);
        } else {
            suppressed += 1;
            if suppressed_timestamp.is_none() {
                suppressed_timestamp = event.timestamp.clone();
            }
        }
    }

    if suppressed > 0 {
        summary.push(EventSummaryItem {
            timestamp: suppressed_timestamp,
            category: "suppressed".to_owned(),
            detail: format!("suppressed {suppressed} low-signal events"),
        });
    }

    summary
}

fn summarize_event_item(event: &EventItem) -> Option<EventSummaryItem> {
    match (event.kind.as_str(), event.event_type.as_deref()) {
        ("session_meta", _) => {
            event
                .payload
                .get("cwd")
                .and_then(Value::as_str)
                .map(|cwd| EventSummaryItem {
                    timestamp: event.timestamp.clone(),
                    category: "session".to_owned(),
                    detail: format!("started in {cwd}"),
                })
        }
        ("event_msg", Some("user_message" | "agent_message")) => event
            .payload
            .get("message")
            .and_then(Value::as_str)
            .map(|text| EventSummaryItem {
                timestamp: event.timestamp.clone(),
                category: "message".to_owned(),
                detail: one_line(text, 180),
            }),
        ("event_msg", Some("exec_command_end")) => summarize_exec_command_end(event),
        ("response_item", Some("function_call")) => summarize_tool_call(event),
        ("response_item", Some("function_call_output" | "custom_tool_call_output")) => {
            summarize_tool_output_issue(event)
        }
        _ => None,
    }
}

fn summarize_exec_command_end(event: &EventItem) -> Option<EventSummaryItem> {
    let payload = &event.payload;
    let command = extract_exec_command(payload).unwrap_or_else(|| "exec_command".to_owned());
    let exit_code = payload
        .get("exit_code")
        .and_then(Value::as_i64)
        .unwrap_or(0);
    let duration = format_duration(payload).unwrap_or_else(|| "unknown duration".to_owned());
    let mut detail = if exit_code == 0 {
        format!(
            "`{}` completed successfully in {}",
            one_line(&command, 120),
            duration
        )
    } else {
        format!(
            "`{}` failed with exit code {} in {}",
            one_line(&command, 120),
            exit_code,
            duration
        )
    };

    if exit_code != 0 {
        if let Some(stderr) = payload.get("stderr").and_then(Value::as_str) {
            let stderr = stderr.trim();
            if !stderr.is_empty() {
                detail.push_str(&format!(": {}", one_line(stderr, 160)));
            }
        } else if let Some(output) = payload.get("aggregated_output").and_then(Value::as_str) {
            let output = output.trim();
            if !output.is_empty() {
                detail.push_str(&format!(": {}", one_line(output, 160)));
            }
        }
    }

    Some(EventSummaryItem {
        timestamp: event.timestamp.clone(),
        category: if exit_code == 0 {
            "command".to_owned()
        } else {
            "error".to_owned()
        },
        detail,
    })
}

fn summarize_tool_call(event: &EventItem) -> Option<EventSummaryItem> {
    let payload = &event.payload;
    let name = payload.get("name").and_then(Value::as_str)?;

    if name == "write_stdin" || name == "exec_command" {
        return None;
    }

    let detail = if name == "apply_patch" {
        let arguments = payload
            .get("arguments")
            .and_then(Value::as_str)
            .unwrap_or("");
        let targets = extract_patch_targets(arguments);
        if targets.is_empty() {
            "apply_patch".to_owned()
        } else {
            format!("apply_patch {}", targets.join(", "))
        }
    } else {
        format!("{}()", name)
    };

    Some(EventSummaryItem {
        timestamp: event.timestamp.clone(),
        category: "tool".to_owned(),
        detail,
    })
}

fn summarize_tool_output_issue(event: &EventItem) -> Option<EventSummaryItem> {
    let output = event.payload.get("output").and_then(Value::as_str)?;
    let cleaned = sanitize_tool_output_issue(output)?;
    if !looks_like_error_output(&cleaned) {
        return None;
    }

    Some(EventSummaryItem {
        timestamp: event.timestamp.clone(),
        category: "error".to_owned(),
        detail: one_line(&cleaned, 180),
    })
}

fn find_thread(conn: &Connection, session_id: &str) -> Result<ThreadSummary> {
    let exact = query_thread(
        conn,
        "SELECT thread_id, path, started_at, cwd, project_name, summary, originator, cli_version, model_provider, agent_nickname, agent_role, message_count, event_count FROM threads WHERE thread_id = ?1",
        params![session_id],
    )?;
    if let Some(thread) = exact {
        return Ok(thread);
    }

    let prefix = query_thread(
        conn,
        "SELECT thread_id, path, started_at, cwd, project_name, summary, originator, cli_version, model_provider, agent_nickname, agent_role, message_count, event_count FROM threads WHERE thread_id LIKE ?1 ORDER BY started_at DESC LIMIT 1",
        params![format!("{session_id}%")],
    )?;
    if let Some(thread) = prefix {
        return Ok(thread);
    }

    bail!("thread not found: {session_id}")
}

fn query_thread<P>(conn: &Connection, sql: &str, params: P) -> Result<Option<ThreadSummary>>
where
    P: rusqlite::Params,
{
    conn.query_row(sql, params, |row| {
        Ok(ThreadSummary {
            thread_id: row.get(0)?,
            path: row.get(1)?,
            started_at: row.get(2)?,
            cwd: row.get(3)?,
            project_name: row.get(4)?,
            summary: row.get(5)?,
            originator: row.get(6)?,
            cli_version: row.get(7)?,
            model_provider: row.get(8)?,
            agent_nickname: row.get(9)?,
            agent_role: row.get(10)?,
            message_count: row.get::<_, i64>(11)? as usize,
            event_count: row.get::<_, i64>(12)? as usize,
        })
    })
    .optional()
    .map_err(Into::into)
}

fn load_fingerprints(conn: &Connection) -> Result<HashMap<String, ThreadFingerprint>> {
    let mut stmt = conn
        .prepare("SELECT path, thread_id, modified_unix, file_size, parser_version FROM threads")?;
    let rows = stmt.query_map([], |row| {
        Ok((
            row.get::<_, String>(0)?,
            ThreadFingerprint {
                thread_id: row.get(1)?,
                modified_unix: row.get(2)?,
                file_size: row.get(3)?,
                parser_version: row.get(4)?,
            },
        ))
    })?;

    let mut map = HashMap::new();
    for row in rows {
        let (path, fingerprint) = row?;
        map.insert(path, fingerprint);
    }
    Ok(map)
}

fn session_files(sessions_root: &Path) -> Result<Vec<PathBuf>> {
    let mut files = Vec::new();
    for entry in WalkDir::new(sessions_root) {
        let entry = entry?;
        if entry.file_type().is_file()
            && entry
                .path()
                .extension()
                .and_then(|ext| ext.to_str())
                .is_some_and(|ext| ext == "jsonl")
        {
            files.push(entry.path().to_path_buf());
        }
    }
    files.sort();
    Ok(files)
}

fn file_fingerprint(path: &Path) -> Result<ThreadFingerprint> {
    let metadata = fs::metadata(path)
        .with_context(|| format!("failed to read metadata for {}", path.display()))?;
    let modified = metadata
        .modified()
        .with_context(|| format!("failed to read mtime for {}", path.display()))?
        .duration_since(UNIX_EPOCH)
        .map_err(|_| anyhow!("mtime before unix epoch: {}", path.display()))?
        .as_secs() as i64;
    let thread_id = infer_thread_id(path)
        .ok_or_else(|| anyhow!("failed to infer thread id from {}", path.display()))?;
    Ok(ThreadFingerprint {
        thread_id,
        modified_unix: modified,
        file_size: metadata.len() as i64,
        parser_version: PARSER_VERSION,
    })
}

fn replace_thread(
    tx: &Transaction<'_>,
    path: &Path,
    fingerprint: &ThreadFingerprint,
    parsed: &ParsedSession,
) -> Result<()> {
    tx.execute(
        "DELETE FROM threads WHERE thread_id = ?1 OR path = ?2",
        params![parsed.thread.thread_id, path.display().to_string()],
    )?;

    tx.execute(
        r#"
        INSERT INTO threads (
            thread_id, path, modified_unix, file_size, started_at, cwd, project_name, summary,
            originator, cli_version, model_provider, agent_nickname, agent_role, parser_version,
            message_count, event_count, indexed_at
        ) VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9, ?10, ?11, ?12, ?13, ?14, ?15, ?16, datetime('now'))
        "#,
        params![
            parsed.thread.thread_id,
            path.display().to_string(),
            fingerprint.modified_unix,
            fingerprint.file_size,
            parsed.thread.started_at,
            parsed.thread.cwd,
            parsed.thread.project_name,
            parsed.thread.summary,
            parsed.thread.originator,
            parsed.thread.cli_version,
            parsed.thread.model_provider,
            parsed.thread.agent_nickname,
            parsed.thread.agent_role,
            fingerprint.parser_version,
            parsed.messages.len() as i64,
            parsed.events.len() as i64
        ],
    )?;

    let mut message_stmt = tx.prepare(
        "INSERT INTO messages (thread_id, seq, timestamp, role, phase, text, text_lower) VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7)",
    )?;
    for (index, message) in parsed.messages.iter().enumerate() {
        message_stmt.execute(params![
            parsed.thread.thread_id,
            index as i64,
            message.timestamp,
            message.role,
            message.phase,
            message.text,
            message.text.to_lowercase(),
        ])?;
    }

    let mut event_stmt = tx.prepare(
        "INSERT INTO events (thread_id, seq, timestamp, kind, event_type, summary, payload_json) VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7)",
    )?;
    for (index, event) in parsed.events.iter().enumerate() {
        event_stmt.execute(params![
            parsed.thread.thread_id,
            index as i64,
            event.timestamp,
            event.kind,
            event.event_type,
            event.summary,
            serde_json::to_string(&event.payload)?,
        ])?;
    }

    Ok(())
}

fn parse_session_file(path: &Path) -> Result<ParsedSession> {
    let file = File::open(path).with_context(|| format!("failed to open {}", path.display()))?;
    let reader = BufReader::new(file);

    let mut thread_id = infer_thread_id(path).unwrap_or_else(|| path.display().to_string());
    let mut started_at = None;
    let mut cwd = None;
    let mut project_name = None;
    let mut summary = None;
    let mut originator = None;
    let mut cli_version = None;
    let mut model_provider = None;
    let mut agent_nickname = None;
    let mut agent_role = None;
    let mut messages = Vec::new();
    let mut events = Vec::new();

    for line in reader.lines() {
        let line = line?;
        if line.trim().is_empty() {
            continue;
        }

        let root: Value = serde_json::from_str(&line)
            .with_context(|| format!("failed to parse jsonl line in {}", path.display()))?;
        let timestamp = root
            .get("timestamp")
            .and_then(Value::as_str)
            .map(ToOwned::to_owned);
        let kind = root
            .get("type")
            .and_then(Value::as_str)
            .unwrap_or("unknown")
            .to_owned();
        let payload = root.get("payload").cloned().unwrap_or(Value::Null);

        if kind == "session_meta" {
            if let Some(id) = payload.get("id").and_then(Value::as_str) {
                thread_id = id.to_owned();
            }
            started_at = payload
                .get("timestamp")
                .and_then(Value::as_str)
                .map(ToOwned::to_owned)
                .or(started_at);
            cwd = payload
                .get("cwd")
                .and_then(Value::as_str)
                .map(ToOwned::to_owned)
                .or(cwd);
            project_name = cwd
                .as_deref()
                .and_then(|path| Path::new(path).file_name())
                .and_then(|name| name.to_str())
                .map(ToOwned::to_owned);
            originator = payload
                .get("originator")
                .and_then(Value::as_str)
                .map(ToOwned::to_owned)
                .or(originator);
            cli_version = payload
                .get("cli_version")
                .and_then(Value::as_str)
                .map(ToOwned::to_owned)
                .or(cli_version);
            model_provider = payload
                .get("model_provider")
                .and_then(Value::as_str)
                .map(ToOwned::to_owned)
                .or(model_provider);
            agent_nickname = payload
                .get("agent_nickname")
                .and_then(Value::as_str)
                .map(ToOwned::to_owned)
                .or(agent_nickname);
            agent_role = payload
                .get("agent_role")
                .and_then(Value::as_str)
                .map(ToOwned::to_owned)
                .or(agent_role);
        }

        if let Some(message) = extract_message(&payload, timestamp.clone()) {
            if !is_noise_message(&message.role, &message.text) {
                if summary.is_none() && message.role == "user" {
                    summary = Some(one_line(&message.text, 180));
                }
                messages.push(message);
            }
        }

        events.push(EventItem {
            timestamp,
            kind: kind.clone(),
            event_type: extract_event_type(&payload),
            summary: summarize_event(&kind, &payload),
            payload,
        });
    }

    if summary.is_none() {
        summary = messages.first().map(|message| one_line(&message.text, 180));
    }

    Ok(ParsedSession {
        thread: ThreadSummary {
            thread_id,
            path: path.display().to_string(),
            started_at,
            cwd,
            project_name,
            summary,
            originator,
            cli_version,
            model_provider,
            agent_nickname,
            agent_role,
            message_count: messages.len(),
            event_count: events.len(),
        },
        messages,
        events,
    })
}

fn extract_message(payload: &Value, timestamp: Option<String>) -> Option<ThreadMessage> {
    if payload.get("type").and_then(Value::as_str) != Some("message") {
        return None;
    }

    let role = payload.get("role").and_then(Value::as_str)?.to_owned();
    let phase = payload
        .get("phase")
        .and_then(Value::as_str)
        .map(ToOwned::to_owned);
    let text = payload
        .get("content")
        .and_then(Value::as_array)
        .map(|parts| {
            parts
                .iter()
                .filter_map(|part| part.get("text").and_then(Value::as_str))
                .collect::<Vec<_>>()
                .join("\n\n")
        })
        .unwrap_or_default()
        .trim()
        .to_owned();

    if text.is_empty() {
        return None;
    }

    Some(ThreadMessage {
        timestamp,
        role,
        phase,
        text,
        cleaned: false,
        original_chars: None,
    })
}

fn extract_event_type(payload: &Value) -> Option<String> {
    payload
        .get("type")
        .and_then(Value::as_str)
        .map(ToOwned::to_owned)
        .or_else(|| {
            payload
                .get("name")
                .and_then(Value::as_str)
                .map(ToOwned::to_owned)
        })
}

fn summarize_event(kind: &str, payload: &Value) -> Option<String> {
    match kind {
        "session_meta" => payload
            .get("cwd")
            .and_then(Value::as_str)
            .map(|cwd| format!("session_meta {cwd}")),
        "event_msg" => payload
            .get("type")
            .and_then(Value::as_str)
            .and_then(|event_type| match event_type {
                "agent_message" | "user_message" => payload
                    .get("message")
                    .and_then(Value::as_str)
                    .map(|text| one_line(text, 180)),
                _ => Some(event_type.to_owned()),
            }),
        "response_item" => match payload.get("type").and_then(Value::as_str) {
            Some("message") => payload
                .get("content")
                .and_then(Value::as_array)
                .map(|parts| {
                    parts
                        .iter()
                        .filter_map(|part| part.get("text").and_then(Value::as_str))
                        .collect::<Vec<_>>()
                        .join(" ")
                })
                .map(|text| one_line(&text, 180)),
            Some("function_call") => payload
                .get("name")
                .and_then(Value::as_str)
                .map(|name| format!("function_call {name}")),
            Some("function_call_output") | Some("custom_tool_call_output") => payload
                .get("output")
                .and_then(Value::as_str)
                .map(|output| one_line(output, 180)),
            Some(other) => Some(other.to_owned()),
            None => Some("response_item".to_owned()),
        },
        _ => Some(kind.to_owned()),
    }
}

fn infer_thread_id(path: &Path) -> Option<String> {
    let stem = path.file_stem()?.to_str()?;
    if stem.len() >= 36 {
        let suffix = &stem[stem.len() - 36..];
        if suffix.matches('-').count() == 4 {
            return Some(suffix.to_owned());
        }
    }
    None
}

fn is_noise_message(role: &str, text: &str) -> bool {
    let trimmed = text.trim();
    role == "developer"
        || trimmed.starts_with("# AGENTS.md instructions")
        || trimmed.starts_with("<environment_context>")
        || trimmed.starts_with("<turn_aborted>")
        || trimmed.starts_with("<subagent_notification>")
        || trimmed.contains("## Compound Codex Tool Mapping")
        || trimmed.contains("<plugins_instructions>")
        || trimmed.contains("<skills_instructions>")
}

fn compare_resolve_candidates(
    left: &ThreadResolveCandidate,
    right: &ThreadResolveCandidate,
) -> std::cmp::Ordering {
    let left_structural = structural_score(left);
    let right_structural = structural_score(right);
    right_structural
        .cmp(&left_structural)
        .then_with(|| {
            if left_structural > 0 || right_structural > 0 {
                thread_activity(right).cmp(&thread_activity(left))
            } else {
                relevance_score(right).cmp(&relevance_score(left))
            }
        })
        .then_with(|| relevance_score(right).cmp(&relevance_score(left)))
        .then_with(|| thread_activity(right).cmp(&thread_activity(left)))
}

fn structural_score(candidate: &ThreadResolveCandidate) -> usize {
    usize::from(candidate.project_exact) * 8
        + usize::from(candidate.project_contains) * 5
        + usize::from(candidate.cwd_contains) * 4
        + usize::from(candidate.path_contains) * 2
}

fn relevance_score(candidate: &ThreadResolveCandidate) -> usize {
    usize::from(candidate.thread_id_contains) * 5
        + usize::from(candidate.summary_contains) * 3
        + candidate.match_count.min(8)
}

fn thread_activity(candidate: &ThreadResolveCandidate) -> &str {
    candidate
        .last_match_at
        .as_deref()
        .or(candidate.started_at.as_deref())
        .unwrap_or("")
}

fn resolve_sort_reason(candidate: &ThreadResolveCandidate) -> String {
    let mut reasons = Vec::new();
    if candidate.project_exact {
        reasons.push("exact project");
    } else if candidate.project_contains {
        reasons.push("project match");
    }
    if candidate.cwd_contains {
        reasons.push("cwd match");
    }
    if candidate.thread_id_contains {
        reasons.push("thread id match");
    }
    if candidate.summary_contains {
        reasons.push("summary match");
    }
    if candidate.match_count > 0 {
        reasons.push("message hits");
    }
    if reasons.is_empty() {
        reasons.push("recent");
    }
    reasons.join(", ")
}

fn clean_thread_message(mut message: ThreadMessage, max_message_chars: usize) -> ThreadMessage {
    let original_chars = message.text.chars().count();
    let trimmed = message.text.trim();

    let replacement = if trimmed.starts_with("<skill>") {
        let name = extract_tag(trimmed, "name");
        let path = extract_tag(trimmed, "path");
        Some(match (name, path) {
            (Some(name), Some(path)) => format!("[skill payload omitted: {name} | {path}]"),
            (Some(name), None) => format!("[skill payload omitted: {name}]"),
            _ => "[skill payload omitted]".to_owned(),
        })
    } else if looks_like_pasted_context(trimmed) {
        Some(format!(
            "{}\n\n[pasted context collapsed: {} chars]",
            one_line(trimmed, 280),
            original_chars
        ))
    } else if original_chars > max_message_chars {
        Some(format!(
            "{}\n\n[truncated from {} chars]",
            one_line(trimmed, max_message_chars.min(600)),
            original_chars
        ))
    } else {
        None
    };

    if let Some(text) = replacement {
        message.text = text;
        message.cleaned = true;
        message.original_chars = Some(original_chars);
    }

    message
}

fn clean_search_text(text: &str, max_message_chars: usize) -> String {
    clean_thread_message(
        ThreadMessage {
            timestamp: None,
            role: "user".to_owned(),
            phase: None,
            text: text.to_owned(),
            cleaned: false,
            original_chars: None,
        },
        max_message_chars,
    )
    .text
}

fn is_substantive_message(text: &str) -> bool {
    let normalized = normalize_summary_value(text);
    if normalized.is_empty() {
        return false;
    }
    if normalized.starts_with("[skill payload omitted")
        || normalized.starts_with("[pasted context collapsed")
        || normalized.starts_with("[truncated from")
    {
        return false;
    }
    if matches!(
        normalized.as_str(),
        "ok" | "okay"
            | "可以"
            | "好的"
            | "推进"
            | "继续"
            | "继续完善"
            | "好的推进"
            | "好的继续"
            | "可以继续"
            | "可以继续完善"
            | "ok我们再使用这个看看效果呢"
    ) {
        return false;
    }

    normalized.chars().count() > 8 || normalized.split_whitespace().count() > 2
}

fn normalize_summary_value(text: &str) -> String {
    compact_whitespace(text)
        .trim_matches(|c: char| {
            c.is_whitespace() || matches!(c, '，' | ',' | '。' | '.' | '！' | '!' | '?' | '？')
        })
        .to_lowercase()
}

fn push_unique(items: &mut Vec<String>, value: String, limit: usize) {
    let normalized = normalize_summary_value(&value);
    if items
        .iter()
        .any(|existing| normalize_summary_value(existing) == normalized)
    {
        return;
    }
    if items.len() < limit {
        items.push(value);
    }
}

fn build_insight_title(digest: &ThreadDigest) -> String {
    if let Some(objective) = &digest.objective {
        let normalized = normalize_summary_value(objective);
        if normalized.contains("cli") {
            return "Insight: A threads CLI becomes useful when retrieval is clean and routine"
                .to_owned();
        }
        return format!("Insight: {}", one_line(objective, 72));
    }

    "Insight: Thread review".to_owned()
}

fn build_insight_summary(digest: &ThreadDigest) -> String {
    match (&digest.objective, &digest.outcome) {
        (Some(objective), Some(outcome)) => format!(
            "{} The thread ended with: {}",
            one_line(objective, 160),
            one_line(outcome, 180)
        ),
        (Some(objective), None) => one_line(objective, 220),
        (None, Some(outcome)) => one_line(outcome, 220),
        (None, None) => "No strong summary signal found in this thread.".to_owned(),
    }
}

fn derive_insight_points(digest: &ThreadDigest) -> Vec<String> {
    let corpus = insight_corpus(digest);
    let normalized = normalize_summary_value(&corpus);
    let mut insights = Vec::new();

    if normalized.contains("recent") || normalized.contains("resolve") {
        push_unique(
            &mut insights,
            "Recency needs to be a first-class retrieval primitive; search and resolve alone are not enough for day-to-day use.".to_owned(),
            5,
        );
    }
    if normalized.contains("clean")
        || normalized.contains("折叠")
        || normalized.contains("pasted context")
        || normalized.contains("skill payload omitted")
        || normalized.contains("噪声")
    {
        push_unique(
            &mut insights,
            "A thread archive only becomes reusable once pasted context and framework noise are collapsed by default.".to_owned(),
            5,
        );
    }
    if normalized.contains("snippet")
        || normalized.contains("messages search")
        || normalized.contains("search")
    {
        push_unique(
            &mut insights,
            "Message search should return snippets, not full messages, so the result list stays scannable under real workloads.".to_owned(),
            5,
        );
    }
    if normalized.contains("events summary")
        || normalized.contains("events read")
        || normalized.contains("event stream")
    {
        push_unique(
            &mut insights,
            "Raw event streams are too noisy for reflection; a summary layer should preserve commands, edits, and failures while suppressing operational chatter.".to_owned(),
            5,
        );
    }
    if normalized.contains("summarize")
        || normalized.contains("复盘")
        || normalized.contains("insight")
    {
        push_unique(
            &mut insights,
            "A reusable threads CLI needs a final interpretation layer that turns operational traces into a readable conclusion, not just searchable logs.".to_owned(),
            5,
        );
    }
    if normalized.contains("github repo") || normalized.contains("开源") {
        push_unique(
            &mut insights,
            "Open-sourcing the tool forced the interface to become clearer: commands, JSON shape, and docs all had to survive real reuse outside the original thread.".to_owned(),
            5,
        );
    }

    if insights.is_empty() {
        push_unique(
            &mut insights,
            "The main pattern in this thread is that useful memory tools need interpretation, not just storage and retrieval.".to_owned(),
            5,
        );
    }

    insights
}

fn build_insight_evidence(digest: &ThreadDigest) -> Vec<String> {
    let mut evidence = Vec::new();

    for request in digest.key_requests.iter().take(3) {
        push_unique(
            &mut evidence,
            format!("User request: {}", one_line(request, 180)),
            6,
        );
    }

    for action in digest
        .key_actions
        .iter()
        .rev()
        .take(3)
        .collect::<Vec<_>>()
        .into_iter()
        .rev()
    {
        push_unique(
            &mut evidence,
            format!("Execution: {}", one_line(&action.detail, 180)),
            6,
        );
    }

    if let Some(outcome) = &digest.outcome {
        push_unique(
            &mut evidence,
            format!("Outcome: {}", one_line(outcome, 180)),
            6,
        );
    }

    evidence
}

fn derive_next_steps(digest: &ThreadDigest, insights: &[String]) -> Vec<String> {
    let normalized = normalize_summary_value(&insight_corpus(digest));
    let mut next_steps = Vec::new();

    if normalized.contains("insight") || normalized.contains("聊天记录") {
        push_unique(
            &mut next_steps,
            "Add a recent-threads batch mode that generates one insight note per thread."
                .to_owned(),
            4,
        );
    }
    if normalized.contains("开源") || normalized.contains("github repo") {
        push_unique(
            &mut next_steps,
            "Add markdown export so an insight can be dropped directly into a README, issue, or changelog.".to_owned(),
            4,
        );
    }
    if insights
        .iter()
        .any(|item| item.contains("interpretation layer"))
    {
        push_unique(
            &mut next_steps,
            "Add optional frontmatter tags like project, theme, and outcome to make insights easier to archive.".to_owned(),
            4,
        );
    }
    if next_steps.is_empty() {
        push_unique(
            &mut next_steps,
            "Add a batch mode so this insight format can be applied to recent threads without resolving one session at a time.".to_owned(),
            4,
        );
    }

    next_steps
}

fn render_insight_content(
    title: &str,
    summary: &str,
    insights: &[String],
    evidence: &[String],
    next_steps: &[String],
) -> String {
    let mut text = String::new();
    writeln!(&mut text, "# {}", title).expect("write to string");
    writeln!(&mut text).expect("write to string");
    writeln!(&mut text, "{}", summary).expect("write to string");

    if !insights.is_empty() {
        writeln!(&mut text).expect("write to string");
        writeln!(&mut text, "## What This Thread Suggests").expect("write to string");
        for insight in insights {
            writeln!(&mut text, "- {}", insight).expect("write to string");
        }
    }

    if !evidence.is_empty() {
        writeln!(&mut text).expect("write to string");
        writeln!(&mut text, "## Evidence").expect("write to string");
        for item in evidence {
            writeln!(&mut text, "- {}", item).expect("write to string");
        }
    }

    if !next_steps.is_empty() {
        writeln!(&mut text).expect("write to string");
        writeln!(&mut text, "## Next").expect("write to string");
        for step in next_steps {
            writeln!(&mut text, "- {}", step).expect("write to string");
        }
    }

    text.trim_end().to_owned()
}

fn insight_corpus(digest: &ThreadDigest) -> String {
    [
        digest.objective.as_deref().unwrap_or(""),
        &digest.key_requests.join(" "),
        &digest
            .key_actions
            .iter()
            .map(|item| item.detail.as_str())
            .collect::<Vec<_>>()
            .join(" "),
        digest.outcome.as_deref().unwrap_or(""),
    ]
    .join(" ")
}

fn extract_exec_command(payload: &Value) -> Option<String> {
    if let Some(cmd) = payload
        .get("parsed_cmd")
        .and_then(Value::as_array)
        .and_then(|items| items.first())
        .and_then(|item| item.get("cmd"))
        .and_then(Value::as_str)
    {
        return Some(cmd.to_owned());
    }

    let command = payload.get("command").and_then(Value::as_array)?;
    if command.len() >= 3
        && command
            .get(1)
            .and_then(Value::as_str)
            .is_some_and(|part| part == "-lc")
    {
        return command
            .get(2)
            .and_then(Value::as_str)
            .map(ToOwned::to_owned);
    }

    Some(
        command
            .iter()
            .filter_map(Value::as_str)
            .collect::<Vec<_>>()
            .join(" "),
    )
}

fn format_duration(payload: &Value) -> Option<String> {
    let duration = payload.get("duration")?;
    let secs = duration.get("secs").and_then(Value::as_i64).unwrap_or(0);
    let nanos = duration.get("nanos").and_then(Value::as_i64).unwrap_or(0);
    let millis = secs.saturating_mul(1000) + nanos.saturating_div(1_000_000);

    if millis >= 1000 {
        Some(format!("{:.1}s", millis as f64 / 1000.0))
    } else {
        Some(format!("{}ms", millis.max(1)))
    }
}

fn extract_patch_targets(arguments: &str) -> Vec<String> {
    let mut targets = Vec::new();

    for line in arguments.lines() {
        for prefix in ["*** Update File: ", "*** Add File: ", "*** Delete File: "] {
            if let Some(target) = line.strip_prefix(prefix) {
                let target = target.trim().to_owned();
                if !target.is_empty() && !targets.contains(&target) {
                    targets.push(target);
                }
            }
        }
    }

    targets
}

fn looks_like_error_output(text: &str) -> bool {
    let normalized = text.to_lowercase();
    if normalized.contains("exited with code 0")
        || normalized.contains("exit code 0")
        || normalized.contains("0 failed")
    {
        return false;
    }
    let head = normalized.chars().take(160).collect::<String>();

    [
        "error:",
        "failed",
        "panic",
        "exception",
        "traceback",
        "not found",
        "permission denied",
        "timed out",
        "unexpected error",
        "💥",
    ]
    .iter()
    .any(|needle| head.contains(needle))
        || normalized.contains("exit code 1")
        || normalized.contains("exited with code 1")
}

fn sanitize_tool_output_issue(text: &str) -> Option<String> {
    let output_only = text
        .split_once("\nOutput:\n")
        .map(|(_, suffix)| suffix)
        .unwrap_or(text);

    let filtered_lines = output_only
        .lines()
        .map(str::trim)
        .filter(|line| !line.is_empty())
        .filter(|line| {
            !line.starts_with("Chunk ID:")
                && !line.starts_with("Wall time:")
                && !line.starts_with("Process exited with code")
                && !line.starts_with("Original token count:")
                && *line != "Output:"
        })
        .filter(|line| !line.starts_with('+'))
        .collect::<Vec<_>>();

    if filtered_lines.is_empty() {
        return None;
    }

    let cleaned = compact_whitespace(&filtered_lines.join(" "));
    if cleaned.is_empty() {
        None
    } else {
        Some(cleaned)
    }
}

fn looks_like_pasted_context(text: &str) -> bool {
    let line_count = text.lines().count();
    text.starts_with("<skill>")
        || text.starts_with("<environment_context>")
        || text.contains("\n---\nname:")
        || text.contains("## Install & Run")
        || text.contains("### Browser-based")
        || (line_count > 80 && text.contains("##"))
}

fn extract_tag(text: &str, tag: &str) -> Option<String> {
    let open = format!("<{tag}>");
    let close = format!("</{tag}>");
    let start = text.find(&open)? + open.len();
    let end = text[start..].find(&close)? + start;
    Some(text[start..end].trim().to_owned())
}

fn normalize_query(query: &str) -> String {
    query.trim().to_lowercase()
}

fn compact_whitespace(text: &str) -> String {
    text.split_whitespace().collect::<Vec<_>>().join(" ")
}

fn one_line(text: &str, max_len: usize) -> String {
    let normalized = compact_whitespace(text);
    if normalized.chars().count() <= max_len {
        normalized
    } else {
        let truncated = normalized
            .chars()
            .take(max_len.saturating_sub(1))
            .collect::<String>();
        format!("{truncated}…")
    }
}

fn escape_html(text: &str) -> String {
    text.replace('&', "&amp;")
        .replace('<', "&lt;")
        .replace('>', "&gt;")
        .replace('"', "&quot;")
        .replace('\'', "&#39;")
}

fn build_search_snippet(text: &str, query: &str, max_len: usize) -> String {
    let compact = compact_whitespace(text);
    if compact.is_empty() {
        return compact;
    }

    if query.is_empty() {
        return one_line(&compact, max_len);
    }

    let compact_lower = compact.to_lowercase();
    let match_bytes = compact_lower.find(query);
    let total_chars = compact.chars().count();

    let Some(match_byte_start) = match_bytes else {
        return one_line(&compact, max_len);
    };

    let match_char_start = compact_lower[..match_byte_start].chars().count();
    let match_char_len = query.chars().count().max(1);
    if total_chars <= max_len {
        return compact;
    }

    let context_budget = max_len.saturating_sub(match_char_len);
    let leading_context = context_budget / 2;
    let trailing_context = context_budget - leading_context;

    let mut start_char = match_char_start.saturating_sub(leading_context);
    let mut end_char = (match_char_start + match_char_len + trailing_context).min(total_chars);

    if end_char - start_char < max_len {
        start_char = end_char.saturating_sub(max_len);
    }
    if end_char - start_char < max_len {
        end_char = (start_char + max_len).min(total_chars);
    }

    let snippet = slice_chars(&compact, start_char, end_char)
        .trim()
        .to_owned();
    let prefix = if start_char > 0 { "…" } else { "" };
    let suffix = if end_char < total_chars { "…" } else { "" };
    format!("{prefix}{snippet}{suffix}")
}

fn slice_chars(text: &str, start_char: usize, end_char: usize) -> &str {
    if start_char >= end_char {
        return "";
    }

    let start_byte = text
        .char_indices()
        .nth(start_char)
        .map(|(index, _)| index)
        .unwrap_or(text.len());
    let end_byte = text
        .char_indices()
        .nth(end_char)
        .map(|(index, _)| index)
        .unwrap_or(text.len());
    &text[start_byte..end_byte]
}

fn ensure_column_exists(
    conn: &Connection,
    table: &str,
    column: &str,
    definition: &str,
) -> Result<()> {
    let statement = format!("ALTER TABLE {table} ADD COLUMN {column} {definition}");
    if let Err(error) = conn.execute(&statement, []) {
        let message = error.to_string();
        if !message.contains("duplicate column name") {
            return Err(error.into());
        }
    }

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::TempDir;

    fn sample_session_text() -> String {
        [
            r#"{"timestamp":"2026-04-13T04:19:11.380Z","type":"session_meta","payload":{"id":"019d8510-9b67-7ff0-914c-cc313085e394","timestamp":"2026-04-13T04:19:11.340Z","cwd":"/tmp/example-project","originator":"codex-tui","cli_version":"0.120.0","model_provider":"openai"}}"#,
            r#"{"timestamp":"2026-04-13T04:19:12.000Z","type":"response_item","payload":{"type":"message","role":"user","content":[{"type":"input_text","text":"build a CLI that searches Codex threads"}]}}"#,
            r#"{"timestamp":"2026-04-13T04:19:13.000Z","type":"response_item","payload":{"type":"message","role":"assistant","content":[{"type":"output_text","text":"I will create the CLI now."}],"phase":"commentary"}}"#,
            r#"{"timestamp":"2026-04-13T04:19:14.000Z","type":"event_msg","payload":{"type":"agent_message","message":"I am syncing the local archive."}}"#,
        ]
        .join("\n")
    }

    fn write_fixture(root: &Path) -> PathBuf {
        let file = root
            .join("2026")
            .join("04")
            .join("13")
            .join("rollout-2026-04-13T12-19-11-019d8510-9b67-7ff0-914c-cc313085e394.jsonl");
        fs::create_dir_all(file.parent().unwrap()).unwrap();
        fs::write(&file, sample_session_text()).unwrap();
        file
    }

    #[test]
    fn parse_session_extracts_readable_messages() {
        let temp = TempDir::new().unwrap();
        let file = write_fixture(temp.path());
        let parsed = parse_session_file(&file).unwrap();

        assert_eq!(
            parsed.thread.thread_id,
            "019d8510-9b67-7ff0-914c-cc313085e394"
        );
        assert_eq!(parsed.messages.len(), 2);
        assert_eq!(
            parsed.thread.project_name.as_deref(),
            Some("example-project")
        );
        assert_eq!(
            parsed.thread.summary.as_deref(),
            Some("build a CLI that searches Codex threads")
        );
    }

    #[test]
    fn sync_and_search_roundtrip() {
        let temp = TempDir::new().unwrap();
        let sessions_root = temp.path().join("sessions");
        write_fixture(&sessions_root);
        let db_path = temp.path().join("index.sqlite3");
        let mut conn = open_database(&db_path).unwrap();

        let report = sync_index(&mut conn, &sessions_root, &db_path).unwrap();
        assert_eq!(report.scanned, 1);
        assert_eq!(report.indexed, 1);

        let matches = search_messages(&conn, "build a CLI", 10).unwrap();
        assert_eq!(matches.len(), 1);
        assert_eq!(matches[0].thread_id, "019d8510-9b67-7ff0-914c-cc313085e394");
        assert_eq!(
            matches[0].summary.as_deref(),
            Some("build a CLI that searches Codex threads")
        );
        assert_eq!(
            matches[0].snippet,
            "build a CLI that searches Codex threads"
        );

        let threads = resolve_threads(&conn, "example-project", 10).unwrap();
        assert_eq!(threads.len(), 1);
        assert_eq!(threads[0].sort_reason, "exact project, cwd match");

        let recent = recent_threads(
            &conn,
            &RecentThreadsArgs {
                limit: 5,
                cwd: Some("/tmp/example-project".to_owned()),
                project: None,
            },
        )
        .unwrap();
        assert_eq!(recent.len(), 1);

        let (thread, messages) = read_thread(
            &conn,
            "019d8510-9b67-7ff0-914c-cc313085e394",
            None,
            true,
            1600,
        )
        .unwrap();
        assert_eq!(thread.message_count, 2);
        assert_eq!(messages.len(), 2);
    }

    #[test]
    fn clean_thread_message_collapses_skill_payloads() {
        let message = ThreadMessage {
            timestamp: None,
            role: "user".to_owned(),
            phase: None,
            text: "<skill>\n<name>opencli-usage</name>\n<path>/tmp/opencli/SKILL.md</path>\n...</skill>"
                .to_owned(),
            cleaned: false,
            original_chars: None,
        };

        let cleaned = clean_thread_message(message, 1600);
        assert!(cleaned.cleaned);
        assert!(cleaned.text.contains("skill payload omitted"));
        assert!(cleaned.text.contains("opencli-usage"));
    }

    #[test]
    fn search_snippet_centers_the_match() {
        let text = "before context before context before context before context before context build a CLI with clean JSON output and composable commands after context after context after context after context after context";
        let snippet = build_search_snippet(text, "build a cli", 80);

        assert!(snippet.contains("build a CLI with clean JSON output"));
        assert!(snippet.starts_with('…'));
        assert!(snippet.ends_with('…'));
        assert!(snippet.chars().count() <= 82);
    }

    #[test]
    fn clean_search_text_collapses_skill_payloads() {
        let text = "<skill>\n<name>opencli-usage</name>\n<path>/tmp/opencli/SKILL.md</path>\n---\nname: opencli-usage\nlong payload\n</skill>";
        let cleaned = clean_search_text(text, 600);

        assert_eq!(
            cleaned,
            "[skill payload omitted: opencli-usage | /tmp/opencli/SKILL.md]"
        );
    }

    #[test]
    fn summarize_event_stream_suppresses_noise_and_keeps_high_signal_items() {
        let events = vec![
            EventItem {
                timestamp: Some("2026-04-13T05:00:00Z".to_owned()),
                kind: "event_msg".to_owned(),
                event_type: Some("token_count".to_owned()),
                summary: Some("token_count".to_owned()),
                payload: json!({ "type": "token_count" }),
            },
            EventItem {
                timestamp: Some("2026-04-13T05:00:01Z".to_owned()),
                kind: "response_item".to_owned(),
                event_type: Some("function_call".to_owned()),
                summary: Some("function_call apply_patch".to_owned()),
                payload: json!({
                    "type": "function_call",
                    "name": "apply_patch",
                    "arguments": "*** Begin Patch\n*** Update File: src/main.rs\n*** End Patch\n"
                }),
            },
            EventItem {
                timestamp: Some("2026-04-13T05:00:02Z".to_owned()),
                kind: "event_msg".to_owned(),
                event_type: Some("exec_command_end".to_owned()),
                summary: Some("exec_command_end".to_owned()),
                payload: json!({
                    "type": "exec_command_end",
                    "parsed_cmd": [{ "cmd": "cargo test", "type": "unknown" }],
                    "duration": { "secs": 2, "nanos": 250_000_000 },
                    "exit_code": 0
                }),
            },
        ];

        let summary = summarize_event_stream(&events);
        assert_eq!(summary.len(), 3);
        assert_eq!(summary[0].category, "suppressed");
        assert_eq!(summary[1].category, "tool");
        assert_eq!(summary[1].detail, "apply_patch src/main.rs");
        assert_eq!(summary[2].category, "command");
        assert!(summary[2].detail.contains("cargo test"));
    }

    #[test]
    fn summarize_failed_exec_command_as_error() {
        let event = EventItem {
            timestamp: Some("2026-04-13T05:00:00Z".to_owned()),
            kind: "event_msg".to_owned(),
            event_type: Some("exec_command_end".to_owned()),
            summary: Some("exec_command_end".to_owned()),
            payload: json!({
                "type": "exec_command_end",
                "command": ["/bin/zsh", "-lc", "cargo test"],
                "duration": { "secs": 0, "nanos": 450_000_000 },
                "exit_code": 1,
                "stderr": "test suite failed"
            }),
        };

        let item = summarize_event_item(&event).unwrap();
        assert_eq!(item.category, "error");
        assert!(item.detail.contains("exit code 1"));
        assert!(item.detail.contains("test suite failed"));
    }

    #[test]
    fn build_thread_digest_extracts_objective_requests_actions_and_outcome() {
        let thread = ThreadSummary {
            thread_id: "thread-1".to_owned(),
            path: "/tmp/thread.jsonl".to_owned(),
            started_at: Some("2026-04-13T05:00:00Z".to_owned()),
            cwd: Some("/tmp/example-project".to_owned()),
            project_name: Some("example-project".to_owned()),
            summary: Some("build a CLI that searches Codex threads".to_owned()),
            originator: None,
            cli_version: None,
            model_provider: None,
            agent_nickname: None,
            agent_role: None,
            message_count: 4,
            event_count: 3,
        };
        let messages = vec![
            ThreadMessage {
                timestamp: Some("2026-04-13T05:00:01Z".to_owned()),
                role: "user".to_owned(),
                phase: None,
                text: "build a CLI that searches Codex threads".to_owned(),
                cleaned: false,
                original_chars: None,
            },
            ThreadMessage {
                timestamp: Some("2026-04-13T05:00:02Z".to_owned()),
                role: "user".to_owned(),
                phase: None,
                text: "你可以单独开一个新的子文件夹，然后建一个 github repo".to_owned(),
                cleaned: false,
                original_chars: None,
            },
            ThreadMessage {
                timestamp: Some("2026-04-13T05:00:03Z".to_owned()),
                role: "user".to_owned(),
                phase: None,
                text: "可以".to_owned(),
                cleaned: false,
                original_chars: None,
            },
            ThreadMessage {
                timestamp: Some("2026-04-13T05:00:04Z".to_owned()),
                role: "assistant".to_owned(),
                phase: Some("final_answer".to_owned()),
                text: "已经做完。我创建了独立仓库并实现了 CLI。".to_owned(),
                cleaned: false,
                original_chars: None,
            },
        ];
        let events = vec![
            EventSummaryItem {
                timestamp: Some("2026-04-13T05:00:05Z".to_owned()),
                category: "suppressed".to_owned(),
                detail: "suppressed 5 low-signal events".to_owned(),
            },
            EventSummaryItem {
                timestamp: Some("2026-04-13T05:00:06Z".to_owned()),
                category: "command".to_owned(),
                detail: "`cargo test` completed successfully in 2.3s".to_owned(),
            },
            EventSummaryItem {
                timestamp: Some("2026-04-13T05:00:07Z".to_owned()),
                category: "tool".to_owned(),
                detail: "apply_patch src/main.rs".to_owned(),
            },
        ];

        let digest = build_thread_digest(thread, messages, events);
        assert_eq!(
            digest.objective.as_deref(),
            Some("build a CLI that searches Codex threads")
        );
        assert_eq!(digest.key_requests.len(), 1);
        assert!(digest.key_requests[0].contains("新的子文件夹"));
        assert_eq!(digest.key_actions.len(), 2);
        assert_eq!(digest.key_actions[0].category, "command");
        assert_eq!(
            digest.outcome.as_deref(),
            Some("已经做完。我创建了独立仓库并实现了 CLI。")
        );
    }

    #[test]
    fn build_thread_insight_renders_note_content() {
        let digest = ThreadDigest {
            thread: ThreadSummary {
                thread_id: "thread-1".to_owned(),
                path: "/tmp/thread.jsonl".to_owned(),
                started_at: Some("2026-04-13T05:00:00Z".to_owned()),
                cwd: Some("/tmp/example-project".to_owned()),
                project_name: Some("example-project".to_owned()),
                summary: Some("build a CLI that searches Codex threads".to_owned()),
                originator: None,
                cli_version: None,
                model_provider: None,
                agent_nickname: None,
                agent_role: None,
                message_count: 4,
                event_count: 3,
            },
            objective: Some("build a CLI that searches Codex threads".to_owned()),
            key_requests: vec![
                "我们测试一下这个cli，你利用这个cli读取我们最近的聊天看看有哪些可以改进的"
                    .to_owned(),
            ],
            key_actions: vec![EventSummaryItem {
                timestamp: Some("2026-04-13T05:00:06Z".to_owned()),
                category: "command".to_owned(),
                detail: "`messages search` completed successfully in 585ms".to_owned(),
            }],
            outcome: Some(
                "现在效果明显比上一版对了，messages search 已经改成 snippet-first。".to_owned(),
            ),
        };

        let insight = build_thread_insight(digest);
        assert!(insight.title.starts_with("Insight:"));
        assert!(!insight.insights.is_empty());
        assert!(insight.content.contains("## What This Thread Suggests"));
        assert!(insight.content.contains("## Evidence"));
    }

    fn sample_fact(thread_id: &str, project: &str, objective: &str) -> SessionFact {
        SessionFact {
            thread: ThreadSummary {
                thread_id: thread_id.to_owned(),
                path: format!("/tmp/{thread_id}.jsonl"),
                started_at: Some(format!(
                    "2026-04-13T05:00:0{}Z",
                    thread_id.chars().last().unwrap_or('0')
                )),
                cwd: Some(format!("/tmp/{project}")),
                project_name: Some(project.to_owned()),
                summary: Some(objective.to_owned()),
                originator: None,
                cli_version: None,
                model_provider: None,
                agent_nickname: None,
                agent_role: None,
                message_count: 4,
                event_count: 2,
            },
            objective: Some(objective.to_owned()),
            outcome: Some(format!("Finished {objective}")),
            key_requests: vec![format!("Please improve {objective}")],
            key_actions: vec![EventSummaryItem {
                timestamp: Some("2026-04-13T05:00:06Z".to_owned()),
                category: "command".to_owned(),
                detail: "`cargo test` completed successfully in 2.3s".to_owned(),
            }],
            substantive_user_messages: 2,
            cleaned_context_messages: 0,
            command_successes: 1,
            command_failures: 0,
        }
    }

    fn sample_analysis(thread_id: &str, project: &str, objective: &str) -> SessionAnalysis {
        let fact = sample_fact(thread_id, project, objective);
        let facets = build_session_facets(&fact);
        SessionAnalysis { fact, facets }
    }

    #[test]
    fn sanitize_tool_output_issue_drops_chunk_wrapper_noise() {
        let output = "Chunk ID: abcd\nWall time: 0.0000 seconds\nProcess exited with code 1\nOriginal token count: 0\nOutput:\n+ set -euo pipefail\nerror: missing session file\n";
        let cleaned = sanitize_tool_output_issue(output).unwrap();

        assert_eq!(cleaned, "error: missing session file");
    }

    #[test]
    fn summarize_tool_output_issue_ignores_wrapper_without_real_error_text() {
        let event = EventItem {
            timestamp: Some("2026-04-13T05:00:00Z".to_owned()),
            kind: "response_item".to_owned(),
            event_type: Some("function_call_output".to_owned()),
            summary: None,
            payload: json!({
                "output": "Chunk ID: abcd\nWall time: 0.0000 seconds\nProcess exited with code 1\nOriginal token count: 0\nOutput:\n+ set -euo pipefail\n"
            }),
        };

        assert!(summarize_tool_output_issue(&event).is_none());
    }

    #[test]
    fn looks_like_error_output_ignores_long_docs_with_late_failed_word() {
        let text = "docs/project_progress/wiki-maintenance/wiki-branch-convergence-audit-2026-04.md:105 Verified by workflow checks so future branch drift is detected automatically and tracked in CI. Additional planning context follows here for many words before mentioning something failed much later in the document body.";

        assert!(!looks_like_error_output(text));
    }

    #[test]
    fn select_insight_facts_round_robins_projects_for_global_reports() {
        let analyses = vec![
            sample_analysis("1", "mnemo", "mnemo one"),
            sample_analysis("2", "mnemo", "mnemo two"),
            sample_analysis("3", "opensource", "open one"),
            sample_analysis("4", "mnemo", "mnemo three"),
            sample_analysis("5", "physedit", "phys one"),
        ];
        let args = InsightsArgs {
            limit: 4,
            project: None,
            output: None,
            event_limit: 120,
            max_message_chars: 1600,
        };

        let selected = select_insight_analyses(analyses, &args);
        let projects = selected
            .iter()
            .map(|analysis| session_project_name(&analysis.fact))
            .collect::<Vec<_>>();

        assert_eq!(
            projects,
            vec![
                "mnemo".to_owned(),
                "opensource".to_owned(),
                "physedit".to_owned(),
                "mnemo".to_owned()
            ]
        );
    }

    #[test]
    fn build_global_evidence_includes_user_requests() {
        let evidence = build_global_evidence(&[sample_analysis("1", "mnemo", "plan mnemo")]);

        assert!(evidence.iter().any(|item| item.detail.contains("Request:")));
        assert!(
            evidence
                .iter()
                .any(|item| item.title.contains("plan mnemo"))
        );
        assert_eq!(evidence[0].id, stable_evidence_id("1"));
    }

    #[test]
    fn build_global_evidence_uses_thread_bound_stable_ids() {
        let analyses = vec![
            sample_analysis(
                "019d8624-895f-7700-ad9f-1309317e8a2c",
                "mnemo",
                "plan mnemo",
            ),
            sample_analysis(
                "019d85a8-f1ba-7d12-981a-bd0cc773b4c4",
                "opensource",
                "trace receipt",
            ),
        ];

        let evidence = build_global_evidence(&analyses);

        assert_eq!(
            evidence
                .iter()
                .map(|item| item.id.as_str())
                .collect::<Vec<_>>(),
            vec![
                "evidence-019d8624-895f-7700-ad9f-1309317e8a2c",
                "evidence-019d85a8-f1ba-7d12-981a-bd0cc773b4c4",
            ]
        );
    }

    #[test]
    fn build_session_facets_classifies_mode_and_signals() {
        let mut fact = sample_fact(
            "9",
            "codex-threads",
            "fix failing cargo test in insights html",
        );
        fact.outcome = None;
        fact.command_failures = 2;
        fact.command_successes = 0;
        fact.key_actions = vec![EventSummaryItem {
            timestamp: Some("2026-04-13T05:00:06Z".to_owned()),
            category: "error".to_owned(),
            detail: "`cargo test` failed with exit code 101 in 2.3s: assertion failed".to_owned(),
        }];

        let facets = build_session_facets(&fact);
        assert_eq!(facets.primary_mode, SessionMode::Debugging);
        assert!(
            facets
                .friction_signals
                .iter()
                .any(|item| item == "Execution friction")
        );
        assert_eq!(facets.outcome_strength, OutcomeStrength::Weak);
    }

    #[test]
    fn aggregate_session_facets_clusters_cross_project_patterns() {
        let mut a = sample_analysis("1", "mnemo", "implement insights html cards");
        a.fact.cleaned_context_messages = 1;
        a.fact.command_failures = 1;
        a.facets = build_session_facets(&a.fact);

        let mut b = sample_analysis("2", "opensource", "fix failing report renderer");
        b.fact.command_failures = 1;
        b.fact.outcome = None;
        b.facets = build_session_facets(&b.fact);

        let c = sample_analysis("3", "mnemo", "plan handoff for insight parity");
        let d = sample_analysis("4", "physedit", "review regression risks in search");

        let analyses = vec![a, b, c, d];
        let evidence = build_global_evidence(&analyses);
        let aggregated = aggregate_session_facets(&analyses, &build_evidence_index(&evidence));
        assert_eq!(aggregated.active_projects[0].name, "mnemo");
        assert!(
            aggregated
                .recurring_frictions
                .iter()
                .any(|item| item.label == "Execution friction" && item.count >= 2)
        );
        assert!(!aggregated.dominant_modes.is_empty());
        assert!(
            aggregated
                .recurring_frictions
                .iter()
                .any(|item| !item.evidence_ids.is_empty())
        );
    }

    #[test]
    fn generate_insights_html_escapes_and_renders_navigation() {
        let analyses = vec![
            sample_analysis("1", "mnemo", "plan <script> report"),
            sample_analysis("2", "opensource", "implement html cards"),
        ];
        let aggregated = aggregate_session_facets(
            &analyses,
            &build_evidence_index(&build_global_evidence(&analyses)),
        );
        let work_areas = build_work_areas(&aggregated);
        let interaction_style = build_interaction_style(&aggregated, &work_areas);
        let what_works = build_what_works(&aggregated, &work_areas);
        let friction = build_friction_cards(&aggregated);
        let suggestions = build_suggestion_cards(&aggregated, &work_areas, &friction);
        let on_the_horizon = build_horizon_items(&aggregated, &work_areas);
        let evidence = vec![EvidenceItem {
            id: stable_evidence_id("1"),
            thread_id: "1".to_owned(),
            title: "plan <script> report".to_owned(),
            detail: "Request: tighten <b>html</b> report".to_owned(),
            project: "mnemo&co".to_owned(),
            mode: "Planning".to_owned(),
            themes: vec!["Reporting & insights".to_owned()],
            confidence: "medium".to_owned(),
        }];
        let metadata = InsightsMetadata {
            sessions_scanned: 7,
            sessions_analyzed: 2,
            project_filter: None,
            generated_report_path: "/tmp/report&<test>.html".to_owned(),
        };
        let trace_aggregated =
            aggregate_session_facets(&analyses, &build_evidence_index(&evidence));
        let briefing = build_insights_briefing(
            &metadata,
            &trace_aggregated,
            &evidence,
            &[
                ExampleSession {
                    thread_id: "1".to_owned(),
                    project: "mnemo".to_owned(),
                    mode: "Planning".to_owned(),
                    mode_confidence: "high".to_owned(),
                    themes: vec!["Reporting & insights".to_owned()],
                    objective: Some("plan <script> report".to_owned()),
                    outcome: None,
                    outcome_strength: "Partial outcome".to_owned(),
                    outcome_confidence: "medium".to_owned(),
                    evidence_ids: vec![stable_evidence_id("1")],
                    classification_notes: vec![
                        "Mode classified from planning keywords.".to_owned(),
                    ],
                },
                ExampleSession {
                    thread_id: "2".to_owned(),
                    project: "opensource".to_owned(),
                    mode: "Implementation".to_owned(),
                    mode_confidence: "high".to_owned(),
                    themes: vec!["Frontend & HTML".to_owned()],
                    objective: Some("implement html cards".to_owned()),
                    outcome: Some("done".to_owned()),
                    outcome_strength: "Strong outcome".to_owned(),
                    outcome_confidence: "high".to_owned(),
                    evidence_ids: vec![],
                    classification_notes: vec![
                        "Outcome classified from successful delivery.".to_owned(),
                    ],
                },
            ],
        );
        let at_a_glance = build_at_a_glance(
            &trace_aggregated,
            &work_areas,
            &interaction_style,
            &friction,
            &suggestions,
        );
        let heuristic_draft = HeuristicDraft {
            at_a_glance: at_a_glance.clone(),
            work_areas: work_areas.clone(),
            interaction_style: interaction_style.clone(),
            what_works: what_works.clone(),
            friction: friction.clone(),
            suggestions: suggestions.clone(),
            on_the_horizon: on_the_horizon.clone(),
            evidence: evidence.clone(),
            content: String::new(),
        };
        let report = InsightsReport {
            metadata: metadata.clone(),
            aggregated,
            briefing,
            source_contract: build_source_contract(),
            trace: build_insights_trace(&metadata, &trace_aggregated, &evidence),
            heuristic_draft,
            at_a_glance,
            work_areas,
            interaction_style,
            what_works,
            friction,
            suggestions,
            on_the_horizon,
            evidence,
            content: String::new(),
        };

        let html = generate_insights_html(&report);
        assert!(html.contains("id=\"section-friction\""));
        assert!(html.contains("href=\"#section-friction\""));
        assert!(html.contains("&lt;script&gt;"));
        assert!(html.contains("/tmp/report&amp;&lt;test&gt;.html"));
    }

    #[test]
    fn at_a_glance_uses_full_analysis_count_not_example_sample() {
        let analyses = (0..12)
            .map(|index| {
                sample_analysis(
                    &(index + 1).to_string(),
                    if index % 2 == 0 {
                        "mnemo"
                    } else {
                        "opensource"
                    },
                    &format!("implement insights flow {index}"),
                )
            })
            .collect::<Vec<_>>();
        let evidence = build_global_evidence(&analyses);
        let aggregated = aggregate_session_facets(&analyses, &build_evidence_index(&evidence));
        let work_areas = build_work_areas(&aggregated);
        let interaction_style = build_interaction_style(&aggregated, &work_areas);
        let friction = build_friction_cards(&aggregated);
        let suggestions = build_suggestion_cards(&aggregated, &work_areas, &friction);
        let glance = build_at_a_glance(
            &aggregated,
            &work_areas,
            &interaction_style,
            &friction,
            &suggestions,
        );

        assert_eq!(aggregated.analyzed_session_count, 12);
        assert_eq!(aggregated.example_sessions.len(), 8);
        assert!(glance.whats_working.contains("12 analyzed sessions"));
    }

    #[test]
    fn build_insights_briefing_exposes_backend_payload() {
        let mut context_heavy = sample_analysis("1", "mnemo", "implement insights backend");
        context_heavy.fact.cleaned_context_messages = 1;
        context_heavy.facets = build_session_facets(&context_heavy.fact);
        let analyses = vec![
            context_heavy,
            sample_analysis("2", "opensource", "plan report generation"),
            sample_analysis("3", "mnemo", "review repeated friction"),
        ];
        let evidence = build_global_evidence(&analyses);
        let aggregated = aggregate_session_facets(&analyses, &build_evidence_index(&evidence));
        let metadata = InsightsMetadata {
            sessions_scanned: 9,
            sessions_analyzed: analyses.len(),
            project_filter: None,
            generated_report_path: "/tmp/report.html".to_owned(),
        };
        let briefing = build_insights_briefing(
            &metadata,
            &aggregated,
            &evidence,
            &aggregated.example_sessions,
        );

        assert_eq!(briefing.schema_version, "insights-briefing-v1");
        assert_eq!(briefing.summary.sessions_analyzed, 3);
        assert!(briefing.summary.example_sessions_are_samples);
        assert_eq!(briefing.summary.example_session_count, 3);
        assert_eq!(
            briefing.summary.example_session_cap,
            EXAMPLE_SESSION_SAMPLE_CAP
        );
        assert!(!briefing.summary.evidence_items_are_samples);
        assert_eq!(briefing.summary.evidence_item_count, 3);
        assert_eq!(
            briefing.summary.evidence_item_cap,
            EXAMPLE_SESSION_SAMPLE_CAP
        );
        assert_eq!(
            briefing.patterns.active_projects.len(),
            aggregated.active_projects.len()
        );
        assert!(!briefing.modeling_notes.is_empty());
        assert!(!briefing.uncertainties.is_empty());
        assert!(briefing.modeling_notes[0].contains("data.heuristic_draft.content"));
        assert!(
            briefing
                .patterns
                .recurring_frictions
                .iter()
                .all(|item| !item.confidence.is_empty())
        );
        assert!(
            briefing
                .example_sessions
                .iter()
                .all(|item| !item.classification_notes.is_empty())
        );
        assert!(
            briefing
                .evidence
                .iter()
                .all(|item| !item.id.is_empty() && !item.confidence.is_empty())
        );
    }

    #[test]
    fn insights_report_json_exposes_contract_and_hides_internal_sections() {
        let analyses = vec![
            sample_analysis("1", "mnemo", "plan insights backend"),
            sample_analysis("2", "opensource", "implement trace receipt"),
            sample_analysis("3", "mnemo", "review sample-only semantics"),
        ];
        let report =
            build_global_insights_report(&analyses, 9, None, Path::new("/tmp/report.html"));
        let json = serde_json::to_value(&report).expect("serialize report");
        let object = json.as_object().expect("report json object");

        for key in [
            "metadata",
            "aggregated",
            "briefing",
            "source_contract",
            "trace",
            "heuristic_draft",
        ] {
            assert!(object.contains_key(key), "missing serialized key: {key}");
        }

        for hidden_key in [
            "at_a_glance",
            "work_areas",
            "interaction_style",
            "what_works",
            "friction",
            "suggestions",
            "on_the_horizon",
            "evidence",
            "content",
        ] {
            assert!(
                !object.contains_key(hidden_key),
                "internal key should be skipped: {hidden_key}"
            );
        }

        assert_eq!(
            json["source_contract"]["canonical_sources"][0],
            "data.source_contract"
        );
        assert_eq!(
            json["source_contract"]["canonical_sources"][1],
            "data.trace"
        );
        assert_eq!(
            json["source_contract"]["derived_overview_sources"][0],
            "data.metadata"
        );
        assert_eq!(
            json["source_contract"]["derived_overview_sources"][1],
            "data.aggregated"
        );
        assert_eq!(
            json["trace"]["canonical_receipt"]["example_sessions_are_samples"],
            true
        );
        assert_eq!(
            json["trace"]["canonical_receipt"]["example_session_cap"],
            EXAMPLE_SESSION_SAMPLE_CAP as u64
        );
        assert_eq!(
            json["trace"]["canonical_receipt"]["evidence_item_cap"],
            EXAMPLE_SESSION_SAMPLE_CAP as u64
        );
        assert!(
            json["source_contract"]["source_notes"][0]
                .as_str()
                .expect("source note")
                .contains("machine-readable contract")
        );
        assert!(json["heuristic_draft"].is_object());
    }

    #[test]
    fn trace_receipt_matches_pattern_and_evidence_payloads() {
        let analyses = vec![
            sample_analysis("1", "mnemo", "implement insights backend"),
            sample_analysis("2", "opensource", "plan report generation"),
            sample_analysis("3", "mnemo", "review repeated friction"),
        ];
        let report =
            build_global_insights_report(&analyses, 11, None, Path::new("/tmp/report.html"));

        let receipt = &report.trace.canonical_receipt;
        let evidence_ids = report
            .trace
            .evidence_receipt
            .iter()
            .map(|item| item.id.as_str())
            .collect::<Vec<_>>();
        let success_labels = report
            .trace
            .recurring_success_patterns
            .iter()
            .map(|item| item.label.as_str())
            .collect::<Vec<_>>();
        let friction_labels = report
            .trace
            .recurring_frictions
            .iter()
            .map(|item| item.label.as_str())
            .collect::<Vec<_>>();

        assert_eq!(receipt.sessions_scanned, 11);
        assert_eq!(receipt.sessions_analyzed, analyses.len());
        assert!(receipt.example_sessions_are_samples);
        assert_eq!(
            receipt.example_session_count,
            report.aggregated.example_sessions.len()
        );
        assert_eq!(receipt.example_session_cap, EXAMPLE_SESSION_SAMPLE_CAP);
        assert_eq!(receipt.evidence_item_count, report.briefing.evidence.len());
        assert_eq!(receipt.evidence_item_cap, EXAMPLE_SESSION_SAMPLE_CAP);
        assert_eq!(
            receipt
                .evidence_ids
                .iter()
                .map(String::as_str)
                .collect::<Vec<_>>(),
            evidence_ids
        );
        assert_eq!(
            receipt
                .recurring_success_labels
                .iter()
                .map(String::as_str)
                .collect::<Vec<_>>(),
            success_labels
        );
        assert_eq!(
            receipt
                .recurring_friction_labels
                .iter()
                .map(String::as_str)
                .collect::<Vec<_>>(),
            friction_labels
        );
    }

    #[test]
    fn canonical_receipt_discloses_evidence_sampling_and_preserves_example_linkage() {
        let analyses = (0..10)
            .map(|index| {
                sample_analysis(
                    &(index + 1).to_string(),
                    if index % 2 == 0 {
                        "mnemo"
                    } else {
                        "opensource"
                    },
                    &format!("session {}", index + 1),
                )
            })
            .collect::<Vec<_>>();
        let report =
            build_global_insights_report(&analyses, 22, None, Path::new("/tmp/report.html"));
        let receipt = &report.trace.canonical_receipt;
        let evidence_ids = report
            .briefing
            .evidence
            .iter()
            .map(|item| item.id.as_str())
            .collect::<Vec<_>>();

        assert!(receipt.example_sessions_are_samples);
        assert!(receipt.evidence_items_are_samples);
        assert_eq!(receipt.example_session_count, EXAMPLE_SESSION_SAMPLE_CAP);
        assert_eq!(receipt.evidence_item_count, EXAMPLE_SESSION_SAMPLE_CAP);
        assert_eq!(receipt.evidence_item_cap, EXAMPLE_SESSION_SAMPLE_CAP);
        assert_eq!(
            receipt
                .evidence_ids
                .iter()
                .map(String::as_str)
                .collect::<Vec<_>>(),
            evidence_ids
        );
        assert!(
            report
                .briefing
                .example_sessions
                .iter()
                .all(|session| !session.evidence_ids.is_empty())
        );
        assert!(
            report
                .briefing
                .uncertainties
                .iter()
                .any(|item| item.contains("Evidence items are also capped"))
        );
    }

    #[test]
    fn candidate_threads_for_insights_breaks_timestamp_ties_with_thread_id() {
        let conn = Connection::open_in_memory().expect("open db");
        conn.execute_batch(
            r#"
            CREATE TABLE threads (
                thread_id TEXT PRIMARY KEY,
                path TEXT NOT NULL,
                started_at TEXT,
                cwd TEXT,
                project_name TEXT,
                summary TEXT,
                originator TEXT,
                cli_version TEXT,
                model_provider TEXT,
                agent_nickname TEXT,
                agent_role TEXT,
                message_count INTEGER NOT NULL,
                event_count INTEGER NOT NULL
            );
            "#,
        )
        .expect("create threads table");
        for thread_id in ["thread-a", "thread-b"] {
            conn.execute(
                r#"
                INSERT INTO threads (
                    thread_id, path, started_at, cwd, project_name, summary,
                    originator, cli_version, model_provider, agent_nickname,
                    agent_role, message_count, event_count
                ) VALUES (?1, ?2, ?3, ?4, ?5, ?6, NULL, NULL, NULL, NULL, NULL, 1, 1)
                "#,
                params![
                    thread_id,
                    format!("/tmp/{thread_id}.jsonl"),
                    "2026-04-13T05:00:00Z",
                    "/tmp/opensource",
                    "opensource",
                    format!("summary for {thread_id}")
                ],
            )
            .expect("insert thread");
        }

        let args = InsightsArgs {
            limit: 2,
            project: None,
            output: None,
            event_limit: 120,
            max_message_chars: 1600,
        };

        let threads = candidate_threads_for_insights(&conn, &args).expect("candidate threads");

        assert_eq!(
            threads
                .iter()
                .map(|thread| thread.thread_id.as_str())
                .collect::<Vec<_>>(),
            vec!["thread-b", "thread-a"]
        );
    }
}
