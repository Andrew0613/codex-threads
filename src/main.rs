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

#[derive(Debug, Clone, Serialize)]
struct InsightsMetadata {
    sessions_scanned: usize,
    sessions_analyzed: usize,
    project_filter: Option<String>,
    generated_report_path: String,
}

#[derive(Debug, Clone, Serialize)]
struct AtAGlance {
    whats_working: String,
    whats_hindering: String,
    quick_wins: String,
    ambitious_workflows: String,
}

#[derive(Debug, Clone, Serialize)]
struct WorkArea {
    name: String,
    session_count: usize,
    description: String,
}

#[derive(Debug, Clone, Serialize)]
struct InteractionStyle {
    narrative: String,
    key_pattern: String,
}

#[derive(Debug, Clone, Serialize)]
struct InsightCard {
    title: String,
    detail: String,
}

#[derive(Debug, Clone, Serialize)]
struct FrictionCard {
    category: String,
    detail: String,
    examples: Vec<String>,
}

#[derive(Debug, Clone, Serialize)]
struct SuggestionCard {
    title: String,
    detail: String,
}

#[derive(Debug, Clone, Serialize)]
struct InsightsReport {
    metadata: InsightsMetadata,
    at_a_glance: AtAGlance,
    work_areas: Vec<WorkArea>,
    interaction_style: InteractionStyle,
    what_works: Vec<InsightCard>,
    friction: Vec<FrictionCard>,
    suggestions: Vec<SuggestionCard>,
    on_the_horizon: Vec<String>,
    evidence: Vec<String>,
    content: String,
}

#[derive(Debug, Clone)]
struct SessionFact {
    thread: ThreadSummary,
    objective: Option<String>,
    outcome: Option<String>,
    key_requests: Vec<String>,
    key_actions: Vec<EventSummaryItem>,
    substantive_user_messages: usize,
    cleaned_context_messages: usize,
    command_successes: usize,
    command_failures: usize,
}

#[derive(Debug, Clone)]
struct ThreadResolveCandidate {
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
    project_exact: bool,
    project_contains: bool,
    cwd_contains: bool,
    path_contains: bool,
    summary_contains: bool,
    thread_id_contains: bool,
}

#[derive(Debug, Clone, Serialize)]
struct EventItem {
    timestamp: Option<String>,
    kind: String,
    event_type: Option<String>,
    summary: Option<String>,
    payload: Value,
}

#[derive(Debug, Clone, Serialize, PartialEq, Eq)]
struct EventSummaryItem {
    timestamp: Option<String>,
    category: String,
    detail: String,
}

#[derive(Debug, Clone, Serialize)]
struct SyncReport {
    sessions_root: String,
    db_path: String,
    scanned: usize,
    indexed: usize,
    updated: usize,
    unchanged: usize,
    removed: usize,
}

#[derive(Debug, Clone, Serialize)]
struct DoctorReport {
    sessions_root: String,
    db_path: String,
    sessions_root_exists: bool,
    db_exists: bool,
    thread_count: usize,
    message_count: usize,
    event_count: usize,
    indexed_files: usize,
}

#[derive(Debug, Clone)]
struct ThreadFingerprint {
    thread_id: String,
    modified_unix: i64,
    file_size: i64,
    parser_version: i64,
}

#[derive(Debug)]
struct ParsedSession {
    thread: ThreadSummary,
    messages: Vec<ThreadMessage>,
    events: Vec<EventItem>,
}

fn run(cli: Cli) -> Result<RenderedOutput> {
    let db_path = cli.db.unwrap_or_else(default_db_path);
    let sessions_root = cli.sessions.unwrap_or_else(default_sessions_root);

    let mut conn = open_database(&db_path)?;

    match cli.command {
        Command::Sync => render_sync(sync_index(&mut conn, &sessions_root, &db_path)?),
        Command::Doctor => render_doctor(run_doctor(&conn, &sessions_root, &db_path)?),
        Command::Insights(args) => render_global_insights(generate_global_insights(&conn, &args)?),
        Command::Messages { command } => match command {
            MessagesCommand::Search(args) => {
                render_messages_search(search_messages(&conn, &args.query, args.limit)?)
            }
        },
        Command::Threads { command } => match command {
            ThreadsCommand::Resolve(args) => {
                render_threads_resolve(resolve_threads(&conn, &args.query, args.limit)?)
            }
            ThreadsCommand::Recent(args) => render_threads_recent(recent_threads(&conn, &args)?),
            ThreadsCommand::Read(args) => render_thread_read(read_thread(
                &conn,
                &args.session_id,
                args.limit,
                !args.raw,
                args.max_message_chars,
            )?),
            ThreadsCommand::Summarize(args) => render_thread_summary(summarize_thread(
                &conn,
                &args.session_id,
                args.event_limit,
                args.max_message_chars,
            )?),
            ThreadsCommand::Insight(args) => render_thread_insight(thread_insight(
                &conn,
                &args.session_id,
                args.event_limit,
                args.max_message_chars,
            )?),
        },
        Command::Events { command } => match command {
            EventsCommand::Read(args) => {
                render_events_read(read_events(&conn, &args.session_id, args.limit)?)
            }
            EventsCommand::Summary(args) => {
                render_events_summary(summarize_events(&conn, &args.session_id, args.limit)?)
            }
        },
    }
}

fn render_sync(report: SyncReport) -> Result<RenderedOutput> {
    let text = format!(
        "Scanned: {}\nIndexed: {}\nUpdated: {}\nUnchanged: {}\nRemoved: {}\nDB: {}\nSessions: {}",
        report.scanned,
        report.indexed,
        report.updated,
        report.unchanged,
        report.removed,
        report.db_path,
        report.sessions_root,
    );

    Ok(RenderedOutput {
        text,
        json: json!({ "ok": true, "command": "sync", "data": report }),
    })
}

fn render_doctor(report: DoctorReport) -> Result<RenderedOutput> {
    let text = format!(
        "Sessions root: {} ({})\nDB: {} ({})\nThreads: {}\nMessages: {}\nEvents: {}\nIndexed files: {}",
        report.sessions_root,
        if report.sessions_root_exists {
            "present"
        } else {
            "missing"
        },
        report.db_path,
        if report.db_exists {
            "present"
        } else {
            "missing"
        },
        report.thread_count,
        report.message_count,
        report.event_count,
        report.indexed_files,
    );

    Ok(RenderedOutput {
        text,
        json: json!({ "ok": true, "command": "doctor", "data": report }),
    })
}

fn render_messages_search(results: Vec<SearchMessageResult>) -> Result<RenderedOutput> {
    let mut text = String::new();
    for (index, item) in results.iter().enumerate() {
        if index > 0 {
            text.push('\n');
            text.push('\n');
        }
        writeln!(
            &mut text,
            "{}. [{}] {}",
            index + 1,
            item.role,
            item.thread_id
        )?;
        if let Some(timestamp) = &item.timestamp {
            writeln!(&mut text, "   at: {}", timestamp)?;
        }
        if let Some(cwd) = &item.cwd {
            writeln!(&mut text, "   cwd: {}", cwd)?;
        }
        if let Some(summary) = &item.summary {
            writeln!(&mut text, "   thread: {}", one_line(summary, 180))?;
        }
        writeln!(&mut text, "   match: {}", item.snippet)?;
    }

    if results.is_empty() {
        text.push_str("No matching messages found.");
    }

    Ok(RenderedOutput {
        text,
        json: json!({ "ok": true, "command": "messages.search", "data": results }),
    })
}

fn render_threads_resolve(results: Vec<ThreadResolveResult>) -> Result<RenderedOutput> {
    let mut text = String::new();
    for (index, item) in results.iter().enumerate() {
        if index > 0 {
            text.push('\n');
            text.push('\n');
        }
        writeln!(&mut text, "{}. {}", index + 1, item.thread_id)?;
        if let Some(started_at) = &item.started_at {
            writeln!(&mut text, "   started: {}", started_at)?;
        }
        if let Some(cwd) = &item.cwd {
            writeln!(&mut text, "   cwd: {}", cwd)?;
        }
        writeln!(&mut text, "   matches: {}", item.match_count)?;
        writeln!(&mut text, "   why: {}", item.sort_reason)?;
        if let Some(summary) = &item.summary {
            writeln!(&mut text, "   {}", one_line(summary, 240))?;
        }
    }

    if results.is_empty() {
        text.push_str("No matching threads found.");
    }

    Ok(RenderedOutput {
        text,
        json: json!({ "ok": true, "command": "threads.resolve", "data": results }),
    })
}

fn render_threads_recent(results: Vec<RecentThreadResult>) -> Result<RenderedOutput> {
    let mut text = String::new();
    for (index, item) in results.iter().enumerate() {
        if index > 0 {
            text.push('\n');
            text.push('\n');
        }
        writeln!(&mut text, "{}. {}", index + 1, item.thread_id)?;
        if let Some(started_at) = &item.started_at {
            writeln!(&mut text, "   started: {}", started_at)?;
        }
        if let Some(cwd) = &item.cwd {
            writeln!(&mut text, "   cwd: {}", cwd)?;
        }
        if let Some(summary) = &item.summary {
            writeln!(&mut text, "   {}", one_line(summary, 240))?;
        }
    }

    if results.is_empty() {
        text.push_str("No recent threads found.");
    }

    Ok(RenderedOutput {
        text,
        json: json!({ "ok": true, "command": "threads.recent", "data": results }),
    })
}

fn render_thread_read(payload: (ThreadSummary, Vec<ThreadMessage>)) -> Result<RenderedOutput> {
    let (thread, messages) = payload;
    let mut text = String::new();
    writeln!(&mut text, "Thread: {}", thread.thread_id)?;
    if let Some(started_at) = &thread.started_at {
        writeln!(&mut text, "Started: {}", started_at)?;
    }
    if let Some(cwd) = &thread.cwd {
        writeln!(&mut text, "CWD: {}", cwd)?;
    }
    if let Some(summary) = &thread.summary {
        writeln!(&mut text, "Summary: {}", one_line(summary, 200))?;
    }
    writeln!(&mut text, "Messages: {}", messages.len())?;

    for message in &messages {
        text.push('\n');
        writeln!(
            &mut text,
            "[{}] {}{}",
            message.timestamp.as_deref().unwrap_or("unknown-time"),
            message.role,
            message
                .phase
                .as_ref()
                .map(|phase| format!(" ({phase})"))
                .unwrap_or_default()
        )?;
        if message.cleaned {
            writeln!(
                &mut text,
                "[cleaned{}]",
                message
                    .original_chars
                    .map(|count| format!(" from {count} chars"))
                    .unwrap_or_default()
            )?;
        }
        writeln!(&mut text, "{}", message.text)?;
    }

    Ok(RenderedOutput {
        text,
        json: json!({
            "ok": true,
            "command": "threads.read",
            "data": {
                "thread": thread,
                "messages": messages,
            }
        }),
    })
}

fn render_thread_summary(digest: ThreadDigest) -> Result<RenderedOutput> {
    let mut text = String::new();
    writeln!(&mut text, "Thread: {}", digest.thread.thread_id)?;
    if let Some(started_at) = &digest.thread.started_at {
        writeln!(&mut text, "Started: {}", started_at)?;
    }
    if let Some(cwd) = &digest.thread.cwd {
        writeln!(&mut text, "CWD: {}", cwd)?;
    }
    if let Some(objective) = &digest.objective {
        writeln!(&mut text, "Objective: {}", objective)?;
    }
    if !digest.key_requests.is_empty() {
        writeln!(&mut text, "Requests:")?;
        for request in &digest.key_requests {
            writeln!(&mut text, "- {}", request)?;
        }
    }
    if !digest.key_actions.is_empty() {
        writeln!(&mut text, "Actions:")?;
        for action in &digest.key_actions {
            writeln!(&mut text, "- [{}] {}", action.category, action.detail)?;
        }
    }
    if let Some(outcome) = &digest.outcome {
        writeln!(&mut text, "Outcome: {}", outcome)?;
    }

    Ok(RenderedOutput {
        text,
        json: json!({
            "ok": true,
            "command": "threads.summarize",
            "data": digest,
        }),
    })
}

fn render_thread_insight(insight: ThreadInsight) -> Result<RenderedOutput> {
    Ok(RenderedOutput {
        text: insight.content.clone(),
        json: json!({
            "ok": true,
            "command": "threads.insight",
            "data": insight,
        }),
    })
}

fn render_global_insights(report: InsightsReport) -> Result<RenderedOutput> {
    Ok(RenderedOutput {
        text: report.content.clone(),
        json: json!({
            "ok": true,
            "command": "insights",
            "data": report,
        }),
    })
}

fn render_events_read(payload: (ThreadSummary, Vec<EventItem>)) -> Result<RenderedOutput> {
    let (thread, events) = payload;
    let mut text = String::new();
    writeln!(&mut text, "Thread: {}", thread.thread_id)?;
    writeln!(&mut text, "Events: {}", events.len())?;

    for event in &events {
        text.push('\n');
        writeln!(
            &mut text,
            "[{}] {}{}",
            event.timestamp.as_deref().unwrap_or("unknown-time"),
            event.kind,
            event
                .event_type
                .as_ref()
                .map(|kind| format!(" ({kind})"))
                .unwrap_or_default()
        )?;
        if let Some(summary) = &event.summary {
            writeln!(&mut text, "{}", summary)?;
        }
    }

    Ok(RenderedOutput {
        text,
        json: json!({
            "ok": true,
            "command": "events.read",
            "data": {
                "thread": thread,
                "events": events,
            }
        }),
    })
}

fn render_events_summary(payload: (ThreadSummary, Vec<EventSummaryItem>)) -> Result<RenderedOutput> {
    let (thread, events) = payload;
    let mut text = String::new();
    writeln!(&mut text, "Thread: {}", thread.thread_id)?;
    writeln!(&mut text, "Summary Events: {}", events.len())?;

    for event in &events {
        text.push('\n');
        writeln!(
            &mut text,
            "[{}] {}",
            event.timestamp.as_deref().unwrap_or("unknown-time"),
            event.category
        )?;
        writeln!(&mut text, "{}", event.detail)?;
    }

    Ok(RenderedOutput {
        text,
        json: json!({
            "ok": true,
            "command": "events.summary",
            "data": {
                "thread": thread,
                "events": events,
            }
        }),
    })
}

fn default_db_path() -> PathBuf {
    let home = dirs::home_dir().unwrap_or_else(|| PathBuf::from("."));
    home.join(".codex")
        .join("codex-threads")
        .join("index.sqlite3")
}

fn default_sessions_root() -> PathBuf {
    let home = dirs::home_dir().unwrap_or_else(|| PathBuf::from("."));
    home.join(".codex").join("sessions")
}

fn open_database(db_path: &Path) -> Result<Connection> {
    if let Some(parent) = db_path.parent() {
        fs::create_dir_all(parent)
            .with_context(|| format!("failed to create db directory {}", parent.display()))?;
    }

    let conn = Connection::open(db_path)
        .with_context(|| format!("failed to open database {}", db_path.display()))?;
    conn.execute_batch(
        r#"
        PRAGMA foreign_keys = ON;
        CREATE TABLE IF NOT EXISTS threads (
            thread_id TEXT PRIMARY KEY,
            path TEXT NOT NULL UNIQUE,
            modified_unix INTEGER NOT NULL,
            file_size INTEGER NOT NULL,
            parser_version INTEGER NOT NULL DEFAULT 0,
            started_at TEXT,
            cwd TEXT,
            project_name TEXT,
            summary TEXT,
            originator TEXT,
            cli_version TEXT,
            model_provider TEXT,
            agent_nickname TEXT,
            agent_role TEXT,
            message_count INTEGER NOT NULL DEFAULT 0,
            event_count INTEGER NOT NULL DEFAULT 0,
            indexed_at TEXT NOT NULL
        );
        CREATE INDEX IF NOT EXISTS idx_threads_started_at ON threads(started_at);
        CREATE INDEX IF NOT EXISTS idx_threads_cwd ON threads(cwd);

        CREATE TABLE IF NOT EXISTS messages (
            id INTEGER PRIMARY KEY AUTOINCREMENT,
            thread_id TEXT NOT NULL,
            seq INTEGER NOT NULL,
            timestamp TEXT,
            role TEXT NOT NULL,
            phase TEXT,
            text TEXT NOT NULL,
            text_lower TEXT NOT NULL,
            FOREIGN KEY(thread_id) REFERENCES threads(thread_id) ON DELETE CASCADE
        );
        CREATE INDEX IF NOT EXISTS idx_messages_thread_seq ON messages(thread_id, seq);
        CREATE INDEX IF NOT EXISTS idx_messages_timestamp ON messages(timestamp);

        CREATE TABLE IF NOT EXISTS events (
            id INTEGER PRIMARY KEY AUTOINCREMENT,
            thread_id TEXT NOT NULL,
            seq INTEGER NOT NULL,
            timestamp TEXT,
            kind TEXT NOT NULL,
            event_type TEXT,
            summary TEXT,
            payload_json TEXT NOT NULL,
            FOREIGN KEY(thread_id) REFERENCES threads(thread_id) ON DELETE CASCADE
        );
        CREATE INDEX IF NOT EXISTS idx_events_thread_seq ON events(thread_id, seq);
        CREATE INDEX IF NOT EXISTS idx_events_timestamp ON events(timestamp);
        "#,
    )?;
    ensure_column_exists(
        &conn,
        "threads",
        "parser_version",
        "INTEGER NOT NULL DEFAULT 0",
    )?;

    Ok(conn)
}

fn sync_index(conn: &mut Connection, sessions_root: &Path, db_path: &Path) -> Result<SyncReport> {
    if !sessions_root.exists() {
        bail!("sessions root does not exist: {}", sessions_root.display());
    }

    let indexed = load_fingerprints(conn)?;
    let files = session_files(sessions_root)?;
    let mut seen_paths = HashSet::new();
    let mut report = SyncReport {
        sessions_root: sessions_root.display().to_string(),
        db_path: db_path.display().to_string(),
        scanned: 0,
        indexed: 0,
        updated: 0,
        unchanged: 0,
        removed: 0,
    };

    let tx = conn.transaction()?;

    for file in &files {
        report.scanned += 1;
        let fingerprint = file_fingerprint(file)?;
        let path_key = file.display().to_string();
        seen_paths.insert(path_key.clone());

        if let Some(current) = indexed.get(&path_key) {
            if current.modified_unix == fingerprint.modified_unix
                && current.file_size == fingerprint.file_size
                && current.parser_version == fingerprint.parser_version
            {
                report.unchanged += 1;
                continue;
            }
        }

        let parsed = parse_session_file(file)?;
        replace_thread(&tx, file, &fingerprint, &parsed)?;

        if indexed.contains_key(&path_key) {
            report.updated += 1;
        } else {
            report.indexed += 1;
        }
    }

    for (path, fingerprint) in &indexed {
        if !seen_paths.contains(path) {
            tx.execute(
                "DELETE FROM threads WHERE thread_id = ?1",
                params![fingerprint.thread_id],
            )?;
            report.removed += 1;
        }
    }

    tx.commit()?;
    Ok(report)
}

fn run_doctor(conn: &Connection, sessions_root: &Path, db_path: &Path) -> Result<DoctorReport> {
    let thread_count: usize = conn
        .query_row("SELECT COUNT(*) FROM threads", [], |row| {
            row.get::<_, i64>(0)
        })
        .unwrap_or(0) as usize;
    let message_count: usize = conn
        .query_row("SELECT COUNT(*) FROM messages", [], |row| {
            row.get::<_, i64>(0)
        })
        .unwrap_or(0) as usize;
    let event_count: usize = conn
        .query_row("SELECT COUNT(*) FROM events", [], |row| {
            row.get::<_, i64>(0)
        })
        .unwrap_or(0) as usize;

    Ok(DoctorReport {
        sessions_root: sessions_root.display().to_string(),
        db_path: db_path.display().to_string(),
        sessions_root_exists: sessions_root.exists(),
        db_exists: db_path.exists(),
        thread_count,
        message_count,
        event_count,
        indexed_files: thread_count,
    })
}

fn search_messages(
    conn: &Connection,
    query: &str,
    limit: usize,
) -> Result<Vec<SearchMessageResult>> {
    let normalized = normalize_query(query);
    let snippet_query = compact_whitespace(query).to_lowercase();
    let mut stmt = conn.prepare(
        r#"
        SELECT
            m.thread_id,
            m.timestamp,
            m.role,
            m.phase,
            m.text,
            t.summary,
            t.cwd,
            t.path
        FROM messages m
        JOIN threads t ON t.thread_id = m.thread_id
        WHERE instr(m.text_lower, ?1) > 0
        ORDER BY COALESCE(m.timestamp, t.started_at) DESC
        LIMIT ?2
        "#,
    )?;

    let rows = stmt.query_map(params![normalized, limit as i64], |row| {
        let text: String = row.get(4)?;
        let cleaned_text = clean_search_text(&text, 600);
        let snippet_source = if cleaned_text.to_lowercase().contains(&snippet_query) {
            cleaned_text.as_str()
        } else {
            text.as_str()
        };
        Ok(SearchMessageResult {
            thread_id: row.get(0)?,
            timestamp: row.get(1)?,
            role: row.get(2)?,
            phase: row.get(3)?,
            summary: row.get(5)?,
            snippet: build_search_snippet(snippet_source, &snippet_query, 180),
            cwd: row.get(6)?,
            path: row.get(7)?,
        })
    })?;

    rows.collect::<rusqlite::Result<Vec<_>>>()
        .map_err(Into::into)
}

fn resolve_threads(
    conn: &Connection,
    query: &str,
    limit: usize,
) -> Result<Vec<ThreadResolveResult>> {
    let normalized = normalize_query(query);
    let mut stmt = conn.prepare(
        r#"
        SELECT
            t.thread_id,
            t.started_at,
            t.cwd,
            t.project_name,
            t.summary,
            t.path,
            t.message_count,
            t.event_count,
            SUM(CASE WHEN instr(m.text_lower, ?1) > 0 THEN 1 ELSE 0 END) AS match_count,
            MAX(CASE WHEN instr(m.text_lower, ?1) > 0 THEN m.timestamp ELSE NULL END) AS last_match_at,
            CASE WHEN lower(COALESCE(t.project_name, '')) = ?1 THEN 1 ELSE 0 END AS project_exact,
            CASE WHEN instr(lower(COALESCE(t.project_name, '')), ?1) > 0 THEN 1 ELSE 0 END AS project_contains,
            CASE WHEN instr(lower(COALESCE(t.cwd, '')), ?1) > 0 THEN 1 ELSE 0 END AS cwd_contains,
            CASE WHEN instr(lower(COALESCE(t.path, '')), ?1) > 0 THEN 1 ELSE 0 END AS path_contains,
            CASE WHEN instr(lower(COALESCE(t.summary, '')), ?1) > 0 THEN 1 ELSE 0 END AS summary_contains,
            CASE WHEN instr(lower(t.thread_id), ?1) > 0 THEN 1 ELSE 0 END AS thread_id_contains
        FROM threads t
        LEFT JOIN messages m ON m.thread_id = t.thread_id
        GROUP BY t.thread_id
        HAVING project_exact = 1
            OR project_contains = 1
            OR cwd_contains = 1
            OR path_contains = 1
            OR summary_contains = 1
            OR thread_id_contains = 1
            OR match_count > 0
        "#,
    )?;

    let rows = stmt.query_map(params![normalized], |row| {
        Ok(ThreadResolveCandidate {
            thread_id: row.get(0)?,
            started_at: row.get(1)?,
            cwd: row.get(2)?,
            project_name: row.get(3)?,
            summary: row.get(4)?,
            path: row.get(5)?,
            message_count: row.get::<_, i64>(6)? as usize,
            event_count: row.get::<_, i64>(7)? as usize,
            match_count: row.get::<_, i64>(8)? as usize,
            last_match_at: row.get(9)?,
            project_exact: row.get::<_, i64>(10)? != 0,
            project_contains: row.get::<_, i64>(11)? != 0,
            cwd_contains: row.get::<_, i64>(12)? != 0,
            path_contains: row.get::<_, i64>(13)? != 0,
            summary_contains: row.get::<_, i64>(14)? != 0,
            thread_id_contains: row.get::<_, i64>(15)? != 0,
        })
    })?;

    let mut candidates = rows.collect::<rusqlite::Result<Vec<_>>>()?;
    candidates.sort_by(|left, right| compare_resolve_candidates(left, right));

    Ok(candidates
        .into_iter()
        .take(limit)
        .map(|item| {
            let sort_reason = resolve_sort_reason(&item);
            ThreadResolveResult {
                thread_id: item.thread_id,
                started_at: item.started_at,
                cwd: item.cwd,
                project_name: item.project_name,
                summary: item.summary,
                path: item.path,
                message_count: item.message_count,
                event_count: item.event_count,
                match_count: item.match_count,
                last_match_at: item.last_match_at,
                sort_reason,
            }
        })
        .collect())
}

fn read_thread(
    conn: &Connection,
    session_id: &str,
    limit: Option<usize>,
    clean: bool,
    max_message_chars: usize,
) -> Result<(ThreadSummary, Vec<ThreadMessage>)> {
    let thread = find_thread(conn, session_id)?;
    let sql = if limit.is_some() {
        r#"
        SELECT timestamp, role, phase, text
        FROM messages
        WHERE thread_id = ?1
        ORDER BY seq ASC
        LIMIT ?2
        "#
    } else {
        r#"
        SELECT timestamp, role, phase, text
        FROM messages
        WHERE thread_id = ?1
        ORDER BY seq ASC
        "#
    };

    let mut stmt = conn.prepare(sql)?;
    let messages = if let Some(limit) = limit {
        let rows = stmt.query_map(params![thread.thread_id, limit as i64], |row| {
            Ok(ThreadMessage {
                timestamp: row.get(0)?,
                role: row.get(1)?,
                phase: row.get(2)?,
                text: row.get(3)?,
                cleaned: false,
                original_chars: None,
            })
        })?;
        rows.collect::<rusqlite::Result<Vec<_>>>()?
    } else {
        let rows = stmt.query_map(params![thread.thread_id], |row| {
            Ok(ThreadMessage {
                timestamp: row.get(0)?,
                role: row.get(1)?,
                phase: row.get(2)?,
                text: row.get(3)?,
                cleaned: false,
                original_chars: None,
            })
        })?;
        rows.collect::<rusqlite::Result<Vec<_>>>()?
    };

    let messages = if clean {
        messages
            .into_iter()
            .map(|message| clean_thread_message(message, max_message_chars))
            .collect()
    } else {
        messages
    };

    Ok((thread, messages))
}

fn recent_threads(conn: &Connection, args: &RecentThreadsArgs) -> Result<Vec<RecentThreadResult>> {
    let cwd_filter = args.cwd.as_ref().map(|value| normalize_query(value));
    let project_filter = args.project.as_ref().map(|value| normalize_query(value));
    let mut stmt = conn.prepare(
        r#"
        SELECT thread_id, started_at, cwd, project_name, summary, path, message_count, event_count
        FROM threads
        WHERE (?1 IS NULL OR instr(lower(COALESCE(cwd, '')), ?1) > 0)
          AND (?2 IS NULL OR lower(COALESCE(project_name, '')) = ?2)
        ORDER BY started_at DESC
        LIMIT ?3
        "#,
    )?;
    let rows = stmt.query_map(
        params![cwd_filter, project_filter, args.limit as i64],
        |row| {
            Ok(RecentThreadResult {
                thread_id: row.get(0)?,
                started_at: row.get(1)?,
                cwd: row.get(2)?,
                project_name: row.get(3)?,
                summary: row.get(4)?,
                path: row.get(5)?,
                message_count: row.get::<_, i64>(6)? as usize,
                event_count: row.get::<_, i64>(7)? as usize,
            })
        },
    )?;

    rows.collect::<rusqlite::Result<Vec<_>>>()
        .map_err(Into::into)
}

fn read_events(
    conn: &Connection,
    session_id: &str,
    limit: usize,
) -> Result<(ThreadSummary, Vec<EventItem>)> {
    let thread = find_thread(conn, session_id)?;
    let mut stmt = conn.prepare(
        r#"
        SELECT timestamp, kind, event_type, summary, payload_json
        FROM events
        WHERE thread_id = ?1
        ORDER BY seq DESC
        LIMIT ?2
        "#,
    )?;

    let rows = stmt.query_map(params![thread.thread_id, limit as i64], |row| {
        let payload_json: String = row.get(4)?;
        let payload: Value = serde_json::from_str(&payload_json).unwrap_or(Value::Null);
        Ok(EventItem {
            timestamp: row.get(0)?,
            kind: row.get(1)?,
            event_type: row.get(2)?,
            summary: row.get(3)?,
            payload,
        })
    })?;

    let mut events = rows.collect::<rusqlite::Result<Vec<_>>>()?;
    events.reverse();
    Ok((thread, events))
}

fn summarize_events(
    conn: &Connection,
    session_id: &str,
    limit: usize,
) -> Result<(ThreadSummary, Vec<EventSummaryItem>)> {
    let (thread, events) = read_events(conn, session_id, limit)?;
    Ok((thread, summarize_event_stream(&events)))
}

fn summarize_thread(
    conn: &Connection,
    session_id: &str,
    event_limit: usize,
    max_message_chars: usize,
) -> Result<ThreadDigest> {
    let (thread, messages) = read_thread(conn, session_id, None, true, max_message_chars)?;
    let (_, events) = summarize_events(conn, session_id, event_limit)?;
    Ok(build_thread_digest(thread, messages, events))
}

fn thread_insight(
    conn: &Connection,
    session_id: &str,
    event_limit: usize,
    max_message_chars: usize,
) -> Result<ThreadInsight> {
    let digest = summarize_thread(conn, session_id, event_limit, max_message_chars)?;
    Ok(build_thread_insight(digest))
}

fn build_thread_digest(
    thread: ThreadSummary,
    messages: Vec<ThreadMessage>,
    events: Vec<EventSummaryItem>,
) -> ThreadDigest {
    let objective = messages
        .iter()
        .find(|message| message.role == "user" && is_substantive_message(&message.text))
        .map(|message| one_line(&message.text, 220))
        .or_else(|| thread.summary.as_ref().map(|summary| one_line(summary, 220)));

    let objective_normalized = objective
        .as_ref()
        .map(|text| normalize_summary_value(text))
        .unwrap_or_default();

    let mut key_requests = Vec::new();
    for message in messages.iter().filter(|message| message.role == "user") {
        let line = one_line(&message.text, 220);
        if !is_substantive_message(&line) {
            continue;
        }
        if normalize_summary_value(&line) == objective_normalized {
            continue;
        }
        push_unique(&mut key_requests, line, 4);
    }

    let outcome = messages
        .iter()
        .rev()
        .find(|message| {
            message.role == "assistant"
                && message.phase.as_deref() == Some("final_answer")
                && is_substantive_message(&message.text)
        })
        .or_else(|| {
            messages
                .iter()
                .rev()
                .find(|message| message.role == "assistant" && is_substantive_message(&message.text))
        })
        .map(|message| one_line(&message.text, 280));

    let key_actions = events
        .into_iter()
        .filter(|event| event.category != "suppressed" && event.category != "message")
        .rev()
        .take(6)
        .collect::<Vec<_>>()
        .into_iter()
        .rev()
        .collect();

    ThreadDigest {
        thread,
        objective,
        key_requests,
        key_actions,
        outcome,
    }
}

fn build_thread_insight(digest: ThreadDigest) -> ThreadInsight {
    let title = build_insight_title(&digest);
    let summary = build_insight_summary(&digest);
    let insights = derive_insight_points(&digest);
    let evidence = build_insight_evidence(&digest);
    let next_steps = derive_next_steps(&digest, &insights);
    let content = render_insight_content(&title, &summary, &insights, &evidence, &next_steps);

    ThreadInsight {
        thread: digest.thread,
        title,
        summary,
        insights,
        evidence,
        next_steps,
        content,
    }
}

fn generate_global_insights(conn: &Connection, args: &InsightsArgs) -> Result<InsightsReport> {
    let candidate_threads = candidate_threads_for_insights(conn, args)?;
    let sessions_scanned = candidate_threads.len();
    let mut facts = Vec::new();

    for thread in candidate_threads {
        if let Some(fact) = build_session_fact(conn, thread, args.event_limit, args.max_message_chars)?
        {
            facts.push(fact);
        }
    }

    facts = select_insight_facts(facts, args);

    if facts.is_empty() {
        bail!("no analyzable sessions found for insights");
    }

    let report_path = args
        .output
        .clone()
        .unwrap_or_else(default_insights_report_path);
    let report = build_global_insights_report(&facts, sessions_scanned, args.project.clone(), &report_path);
    write_insights_report_html(&report_path, &report)?;
    Ok(report)
}

fn candidate_threads_for_insights(
    conn: &Connection,
    args: &InsightsArgs,
) -> Result<Vec<ThreadSummary>> {
    let project_filter = args.project.as_ref().map(|value| normalize_query(value));
    let fetch_limit = if args.project.is_some() {
        args.limit
    } else {
        args.limit.saturating_mul(4).clamp(args.limit, 200)
    };
    let mut stmt = conn.prepare(
        r#"
        SELECT thread_id, path, started_at, cwd, project_name, summary, originator, cli_version, model_provider, agent_nickname, agent_role, message_count, event_count
        FROM threads
        WHERE (?1 IS NULL OR lower(COALESCE(project_name, '')) = ?1)
        ORDER BY started_at DESC
        LIMIT ?2
        "#,
    )?;

    let rows = stmt.query_map(params![project_filter, fetch_limit as i64], |row| {
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
    })?;

    rows.collect::<rusqlite::Result<Vec<_>>>()
        .map_err(Into::into)
}

fn build_session_fact(
    conn: &Connection,
    thread: ThreadSummary,
    event_limit: usize,
    max_message_chars: usize,
) -> Result<Option<SessionFact>> {
    let (_, messages) = read_thread(conn, &thread.thread_id, None, true, max_message_chars)?;
    let (_, events) = summarize_events(conn, &thread.thread_id, event_limit)?;

    let substantive_user_messages = messages
        .iter()
        .filter(|message| message.role == "user" && is_substantive_message(&message.text))
        .count();
    let cleaned_context_messages = messages.iter().filter(|message| message.cleaned).count();
    let command_successes = events
        .iter()
        .filter(|event| event.category == "command")
        .count();
    let command_failures = events.iter().filter(|event| event.category == "error").count();

    if substantive_user_messages == 0 {
        return Ok(None);
    }
    if thread.message_count < 3 && command_successes == 0 && command_failures == 0 {
        return Ok(None);
    }

    let digest = build_thread_digest(thread.clone(), messages, events);
    Ok(Some(SessionFact {
        thread,
        objective: digest.objective,
        outcome: digest.outcome,
        key_requests: digest.key_requests,
        key_actions: digest.key_actions,
        substantive_user_messages,
        cleaned_context_messages,
        command_successes,
        command_failures,
    }))
}

fn select_insight_facts(mut facts: Vec<SessionFact>, args: &InsightsArgs) -> Vec<SessionFact> {
    if facts.len() <= args.limit {
        return facts;
    }

    if args.project.is_some() {
        facts.truncate(args.limit);
        return facts;
    }

    let mut buckets: HashMap<String, VecDeque<SessionFact>> = HashMap::new();
    let mut order = Vec::new();
    for fact in facts {
        let project = session_project_name(&fact);
        if !buckets.contains_key(&project) {
            order.push(project.clone());
        }
        buckets.entry(project).or_default().push_back(fact);
    }

    let mut selected = Vec::new();
    while selected.len() < args.limit {
        let mut made_progress = false;
        for project in &order {
            if selected.len() >= args.limit {
                break;
            }
            if let Some(fact) = buckets.get_mut(project).and_then(VecDeque::pop_front) {
                selected.push(fact);
                made_progress = true;
            }
        }

        if !made_progress {
            break;
        }
    }

    selected
}

fn build_global_insights_report(
    facts: &[SessionFact],
    sessions_scanned: usize,
    project_filter: Option<String>,
    report_path: &Path,
) -> InsightsReport {
    let work_areas = build_work_areas(facts);
    let interaction_style = build_interaction_style(facts, &work_areas);
    let what_works = build_what_works(facts, &work_areas);
    let friction = build_friction_cards(facts);
    let suggestions = build_suggestion_cards(facts, &work_areas, &friction);
    let on_the_horizon = build_horizon_items(facts, &work_areas);
    let evidence = build_global_evidence(facts);
    let at_a_glance =
        build_at_a_glance(facts, &work_areas, &interaction_style, &friction, &suggestions);
    let metadata = InsightsMetadata {
        sessions_scanned,
        sessions_analyzed: facts.len(),
        project_filter,
        generated_report_path: report_path.display().to_string(),
    };
    let content = render_global_insights_content(
        &metadata,
        &at_a_glance,
        &work_areas,
        &interaction_style,
        &what_works,
        &friction,
        &suggestions,
        &on_the_horizon,
        &evidence,
    );

    InsightsReport {
        metadata,
        at_a_glance,
        work_areas,
        interaction_style,
        what_works,
        friction,
        suggestions,
        on_the_horizon,
        evidence,
        content,
    }
}

fn build_work_areas(facts: &[SessionFact]) -> Vec<WorkArea> {
    let mut grouped: HashMap<String, Vec<&SessionFact>> = HashMap::new();
    for fact in facts {
        grouped
            .entry(session_project_name(fact))
            .or_default()
            .push(fact);
    }

    let mut areas = grouped
        .into_iter()
        .map(|(name, group)| {
            let mut examples = Vec::new();
            for fact in &group {
                if let Some(objective) = &fact.objective {
                    push_unique(&mut examples, one_line(objective, 120), 2);
                } else if let Some(summary) = &fact.thread.summary {
                    push_unique(&mut examples, one_line(summary, 120), 2);
                }
            }
            let description = if examples.is_empty() {
                format!("{} recent sessions in this area.", group.len())
            } else {
                format!(
                    "{} recent sessions focused on {}.",
                    group.len(),
                    examples.join(" / ")
                )
            };
            WorkArea {
                name,
                session_count: group.len(),
                description,
            }
        })
        .collect::<Vec<_>>();

    areas.sort_by(|left, right| {
        right
            .session_count
            .cmp(&left.session_count)
            .then_with(|| left.name.cmp(&right.name))
    });
    areas.truncate(5);
    areas
}

fn build_interaction_style(facts: &[SessionFact], work_areas: &[WorkArea]) -> InteractionStyle {
    let total = facts.len().max(1);
    let iterative_sessions = facts
        .iter()
        .filter(|fact| fact.substantive_user_messages >= 2)
        .count();
    let command_heavy_sessions = facts
        .iter()
        .filter(|fact| fact.command_successes + fact.command_failures >= 2)
        .count();
    let context_heavy_sessions = facts
        .iter()
        .filter(|fact| fact.cleaned_context_messages > 0)
        .count();
    let dominant_area = work_areas.first().map(|area| area.session_count).unwrap_or(0);
    let dominant_share = dominant_area as f64 / total as f64;

    let mut sentences = Vec::new();
    let key_pattern = if iterative_sessions * 2 >= total {
        "You tend to refine work inside the same thread instead of restarting from scratch."
            .to_owned()
    } else if dominant_share >= 0.6 {
        "You usually stay anchored on one project until the thread produces a concrete result."
            .to_owned()
    } else {
        "You use Codex across multiple threads and projects, then come back to tighten the ones that matter."
            .to_owned()
    };
    sentences.push(key_pattern.clone());

    if command_heavy_sessions * 2 >= total {
        sentences.push(
            "Your sessions are operational, not just conversational: builds, installs, searches, and checks are part of how you validate progress."
                .to_owned(),
        );
    }
    if context_heavy_sessions * 3 >= total {
        sentences.push(
            "You frequently inject dense reference context, which makes cleanup, collapsing, and summary layers important to keep the thread reusable."
                .to_owned(),
        );
    }
    if dominant_share >= 0.6 {
        let area_name = work_areas
            .first()
            .map(|area| area.name.as_str())
            .unwrap_or("one project");
        sentences.push(format!(
            "Recent usage is concentrated around {area_name}, suggesting you prefer going deep on one active system before switching context."
        ));
    } else if work_areas.len() > 1 {
        sentences.push(
            "Recent usage spans multiple work areas, so cross-thread summaries need to separate project-specific patterns from global habits."
                .to_owned(),
        );
    }

    InteractionStyle {
        narrative: sentences.join(" "),
        key_pattern,
    }
}

fn build_what_works(facts: &[SessionFact], work_areas: &[WorkArea]) -> Vec<InsightCard> {
    let total = facts.len().max(1);
    let with_outcomes = facts.iter().filter(|fact| fact.outcome.is_some()).count();
    let command_sessions = facts
        .iter()
        .filter(|fact| fact.command_successes > 0)
        .count();
    let iterative_sessions = facts
        .iter()
        .filter(|fact| fact.substantive_user_messages >= 2)
        .count();

    let mut cards = Vec::new();
    if command_sessions > 0 {
        cards.push(InsightCard {
            title: "You validate work with real commands".to_owned(),
            detail: format!(
                "{} of {} analyzed sessions included successful command execution, which keeps the conversation tied to observable results instead of speculation.",
                command_sessions, total
            ),
        });
    }
    if iterative_sessions > 0 {
        cards.push(InsightCard {
            title: "You improve results by iterating in-thread".to_owned(),
            detail: format!(
                "{} sessions contained substantive follow-up requests, showing that your best outcomes come from tightening the same thread rather than throwing it away.",
                iterative_sessions
            ),
        });
    }
    if with_outcomes > 0 {
        cards.push(InsightCard {
            title: "Threads often end in a concrete deliverable".to_owned(),
            detail: format!(
                "{} analyzed sessions ended with a detectable assistant outcome, which means your history contains reusable conclusions rather than only partial exploration.",
                with_outcomes
            ),
        });
    }
    if cards.is_empty() {
        cards.push(InsightCard {
            title: "Your history already contains reusable signals".to_owned(),
            detail: "Even without explicit success markers everywhere, the session archive still captures repeatable goals, actions, and outcomes that can be turned into guidance.".to_owned(),
        });
    }
    if let Some(area) = work_areas.first() {
        cards.push(InsightCard {
            title: "Project concentration creates stronger memory".to_owned(),
            detail: format!(
                "The busiest recent area is {}, which makes it easier to detect repeated patterns and extract project-specific workflows.",
                area.name
            ),
        });
    }
    cards.truncate(4);
    cards
}

fn build_friction_cards(facts: &[SessionFact]) -> Vec<FrictionCard> {
    let mut cards = Vec::new();
    let failed_sessions = facts
        .iter()
        .filter(|fact| fact.command_failures > 0)
        .collect::<Vec<_>>();
    if !failed_sessions.is_empty() {
        let mut examples = Vec::new();
        for fact in &failed_sessions {
            for action in fact.key_actions.iter().filter(|action| action.category == "error") {
                push_unique(&mut examples, one_line(&action.detail, 160), 3);
            }
        }
        cards.push(FrictionCard {
            category: "Execution friction".to_owned(),
            detail: format!(
                "{} analyzed sessions contained failed commands or tool-level errors, which means environment and validation issues still leak into the workflow.",
                failed_sessions.len()
            ),
            examples,
        });
    }

    let context_heavy_sessions = facts
        .iter()
        .filter(|fact| fact.cleaned_context_messages > 0)
        .collect::<Vec<_>>();
    if !context_heavy_sessions.is_empty() {
        let mut examples = Vec::new();
        for fact in context_heavy_sessions.iter().take(3) {
            if let Some(objective) = &fact.objective {
                push_unique(&mut examples, one_line(objective, 160), 3);
            }
        }
        cards.push(FrictionCard {
            category: "Context overload".to_owned(),
            detail: format!(
                "{} sessions required collapsing pasted context or oversized payloads, which makes raw transcripts harder to reuse without cleanup layers.",
                context_heavy_sessions.len()
            ),
            examples,
        });
    }

    let weak_outcome_sessions = facts
        .iter()
        .filter(|fact| fact.outcome.is_none())
        .count();
    if weak_outcome_sessions * 2 >= facts.len().max(1) {
        cards.push(FrictionCard {
            category: "Soft thread endings".to_owned(),
            detail: "Many sessions do not end with a crisp final-answer outcome, so retrospective analysis has to infer completion from indirect signals.".to_owned(),
            examples: Vec::new(),
        });
    }

    cards.truncate(3);
    cards
}

fn build_suggestion_cards(
    facts: &[SessionFact],
    work_areas: &[WorkArea],
    friction: &[FrictionCard],
) -> Vec<SuggestionCard> {
    let mut suggestions = Vec::new();
    let context_heavy_sessions = facts
        .iter()
        .filter(|fact| fact.cleaned_context_messages > 0)
        .count();
    if context_heavy_sessions > 0 {
        suggestions.push(SuggestionCard {
            title: "Keep context folded by default".to_owned(),
            detail: "Recent usage shows that long pasted references are common. Preserve compact snippets and collapsed payload markers so global reports stay readable.".to_owned(),
        });
    }
    if work_areas.len() > 1 {
        let area = work_areas
            .first()
            .map(|item| item.name.as_str())
            .unwrap_or("project");
        suggestions.push(SuggestionCard {
            title: "Run insights per project when needed".to_owned(),
            detail: format!(
                "Your recent usage spans multiple work areas. Use `codex-threads insights --project {area}` when you want a cleaner project-specific report."
            ),
        });
    }
    if friction.iter().any(|item| item.category == "Execution friction") {
        suggestions.push(SuggestionCard {
            title: "Capture preflight checks for recurring command flows".to_owned(),
            detail: "Failed commands are one of the clearest friction signals. Turning setup checks into a repeatable preflight reduces wasted retries.".to_owned(),
        });
    }
    suggestions.push(SuggestionCard {
        title: "Promote repeated good threads into reusable patterns".to_owned(),
        detail: "When a thread repeatedly follows the same path from request to validation to outcome, preserve that flow as a checklist or automation instead of rediscovering it manually.".to_owned(),
    });
    suggestions.truncate(4);
    suggestions
}

fn build_horizon_items(facts: &[SessionFact], work_areas: &[WorkArea]) -> Vec<String> {
    let mut items = Vec::new();
    if facts.iter().any(|fact| fact.command_successes >= 2) {
        items.push(
            "The next step is not just reading old threads but turning recurring execution loops into batch reports, checklists, or automations."
                .to_owned(),
        );
    }
    if let Some(area) = work_areas.first() {
        items.push(format!(
            "Because {} dominates recent work, a future project-specific report can become a durable working memory layer for that codebase.",
            area.name
        ));
    }
    if items.is_empty() {
        items.push(
            "As the archive grows, the most valuable upgrade is moving from single-thread recap to repeated pattern detection across many threads."
                .to_owned(),
        );
    }
    items
}

fn build_global_evidence(facts: &[SessionFact]) -> Vec<String> {
    let mut evidence = Vec::new();
    for fact in facts.iter().take(6) {
        if let Some(objective) = &fact.objective {
            push_unique(
                &mut evidence,
                format!("Session objective: {}", one_line(objective, 180)),
                8,
            );
        }
        if let Some(request) = fact.key_requests.first() {
            push_unique(
                &mut evidence,
                format!("User request: {}", one_line(request, 180)),
                8,
            );
        }
        if let Some(outcome) = &fact.outcome {
            push_unique(
                &mut evidence,
                format!("Outcome: {}", one_line(outcome, 180)),
                8,
            );
        }
        for action in fact.key_actions.iter().take(2) {
            push_unique(
                &mut evidence,
                format!("Action: {}", one_line(&action.detail, 180)),
                8,
            );
        }
    }
    evidence
}

fn build_at_a_glance(
    facts: &[SessionFact],
    work_areas: &[WorkArea],
    interaction_style: &InteractionStyle,
    friction: &[FrictionCard],
    suggestions: &[SuggestionCard],
) -> AtAGlance {
    let total = facts.len().max(1);
    let with_outcomes = facts.iter().filter(|fact| fact.outcome.is_some()).count();
    let command_sessions = facts
        .iter()
        .filter(|fact| fact.command_successes > 0)
        .count();
    let failed_sessions = facts
        .iter()
        .filter(|fact| fact.command_failures > 0)
        .count();

    let dominant_area = work_areas.first().map(|area| area.name.as_str()).unwrap_or("recent work");
    let hindering = if let Some(card) = friction.first() {
        card.detail.clone()
    } else {
        "The main limitation is that many transcripts still need interpretation before they become reusable guidance.".to_owned()
    };
    let quick_win = suggestions
        .first()
        .map(|item| item.detail.clone())
        .unwrap_or_else(|| "Generate project-scoped reports when you want a tighter view of one active codebase.".to_owned());

    AtAGlance {
        whats_working: format!(
            "{} {} of {} analyzed sessions ended with a detectable outcome, and {} included successful command execution.",
            interaction_style.key_pattern, with_outcomes, total, command_sessions
        ),
        whats_hindering: if failed_sessions > 0 {
            format!("{hindering} Failed execution showed up in {failed_sessions} analyzed sessions.")
        } else {
            hindering
        },
        quick_wins: quick_win,
        ambitious_workflows: format!(
            "The strongest future workflow is a reusable report layer around {dominant_area}: repeated threads can become project memory, not just archived transcripts."
        ),
    }
}

fn render_global_insights_content(
    metadata: &InsightsMetadata,
    at_a_glance: &AtAGlance,
    work_areas: &[WorkArea],
    interaction_style: &InteractionStyle,
    what_works: &[InsightCard],
    friction: &[FrictionCard],
    suggestions: &[SuggestionCard],
    on_the_horizon: &[String],
    evidence: &[String],
) -> String {
    let mut text = String::new();
    writeln!(&mut text, "# Codex Insights").expect("write to string");
    writeln!(
        &mut text,
        "\n{} sessions scanned · {} analyzed · report: {}",
        metadata.sessions_scanned, metadata.sessions_analyzed, metadata.generated_report_path
    )
    .expect("write to string");

    writeln!(&mut text, "\n## At a Glance").expect("write to string");
    writeln!(&mut text, "- **What's working:** {}", at_a_glance.whats_working)
        .expect("write to string");
    writeln!(
        &mut text,
        "- **What's hindering:** {}",
        at_a_glance.whats_hindering
    )
    .expect("write to string");
    writeln!(&mut text, "- **Quick wins:** {}", at_a_glance.quick_wins)
        .expect("write to string");
    writeln!(
        &mut text,
        "- **Ambitious workflows:** {}",
        at_a_glance.ambitious_workflows
    )
    .expect("write to string");

    if !work_areas.is_empty() {
        writeln!(&mut text, "\n## What You Work On").expect("write to string");
        for area in work_areas {
            writeln!(
                &mut text,
                "- **{}**: {}",
                area.name, area.description
            )
            .expect("write to string");
        }
    }

    writeln!(&mut text, "\n## How You Use Codex").expect("write to string");
    writeln!(&mut text, "{}", interaction_style.narrative).expect("write to string");

    if !what_works.is_empty() {
        writeln!(&mut text, "\n## What Works").expect("write to string");
        for item in what_works {
            writeln!(&mut text, "- **{}**: {}", item.title, item.detail)
                .expect("write to string");
        }
    }

    if !friction.is_empty() {
        writeln!(&mut text, "\n## Where Things Go Wrong").expect("write to string");
        for item in friction {
            writeln!(&mut text, "- **{}**: {}", item.category, item.detail)
                .expect("write to string");
            for example in &item.examples {
                writeln!(&mut text, "  - {}", example).expect("write to string");
            }
        }
    }

    if !suggestions.is_empty() {
        writeln!(&mut text, "\n## Suggestions").expect("write to string");
        for item in suggestions {
            writeln!(&mut text, "- **{}**: {}", item.title, item.detail)
                .expect("write to string");
        }
    }

    if !on_the_horizon.is_empty() {
        writeln!(&mut text, "\n## On the Horizon").expect("write to string");
        for item in on_the_horizon {
            writeln!(&mut text, "- {}", item).expect("write to string");
        }
    }

    if !evidence.is_empty() {
        writeln!(&mut text, "\n## Evidence").expect("write to string");
        for item in evidence {
            writeln!(&mut text, "- {}", item).expect("write to string");
        }
    }

    text.trim_end().to_owned()
}

fn write_insights_report_html(path: &Path, report: &InsightsReport) -> Result<()> {
    if let Some(parent) = path.parent() {
        fs::create_dir_all(parent)
            .with_context(|| format!("failed to create report directory {}", parent.display()))?;
    }
    fs::write(path, generate_insights_html(report))
        .with_context(|| format!("failed to write report {}", path.display()))?;
    Ok(())
}

fn generate_insights_html(report: &InsightsReport) -> String {
    let work_areas = report
        .work_areas
        .iter()
        .map(|area| {
            format!(
                "<li><strong>{}</strong>: {}</li>",
                escape_html(&area.name),
                escape_html(&area.description)
            )
        })
        .collect::<Vec<_>>()
        .join("");
    let what_works = report
        .what_works
        .iter()
        .map(|item| {
            format!(
                "<li><strong>{}</strong>: {}</li>",
                escape_html(&item.title),
                escape_html(&item.detail)
            )
        })
        .collect::<Vec<_>>()
        .join("");
    let friction = report
        .friction
        .iter()
        .map(|item| {
            let examples = if item.examples.is_empty() {
                String::new()
            } else {
                format!(
                    "<ul>{}</ul>",
                    item.examples
                        .iter()
                        .map(|example| format!("<li>{}</li>", escape_html(example)))
                        .collect::<Vec<_>>()
                        .join("")
                )
            };
            format!(
                "<li><strong>{}</strong>: {}{}</li>",
                escape_html(&item.category),
                escape_html(&item.detail),
                examples
            )
        })
        .collect::<Vec<_>>()
        .join("");
    let suggestions = report
        .suggestions
        .iter()
        .map(|item| {
            format!(
                "<li><strong>{}</strong>: {}</li>",
                escape_html(&item.title),
                escape_html(&item.detail)
            )
        })
        .collect::<Vec<_>>()
        .join("");
    let horizon = report
        .on_the_horizon
        .iter()
        .map(|item| format!("<li>{}</li>", escape_html(item)))
        .collect::<Vec<_>>()
        .join("");
    let evidence = report
        .evidence
        .iter()
        .map(|item| format!("<li>{}</li>", escape_html(item)))
        .collect::<Vec<_>>()
        .join("");

    format!(
        "<!doctype html><html><head><meta charset=\"utf-8\"><title>Codex Insights</title><style>body{{font-family:ui-sans-serif,system-ui,-apple-system,sans-serif;margin:40px auto;max-width:900px;padding:0 20px;line-height:1.6;color:#1f2937}}h1,h2{{color:#111827}}.meta{{color:#6b7280;margin-bottom:24px}}section{{margin:28px 0}}ul{{padding-left:20px}}.glance li{{margin:8px 0}}code{{background:#f3f4f6;padding:2px 6px;border-radius:6px}}</style></head><body><h1>Codex Insights</h1><div class=\"meta\">{} sessions scanned · {} analyzed · report generated at {}</div><section><h2>At a Glance</h2><ul class=\"glance\"><li><strong>What's working:</strong> {}</li><li><strong>What's hindering:</strong> {}</li><li><strong>Quick wins:</strong> {}</li><li><strong>Ambitious workflows:</strong> {}</li></ul></section><section><h2>What You Work On</h2><ul>{}</ul></section><section><h2>How You Use Codex</h2><p>{}</p></section><section><h2>What Works</h2><ul>{}</ul></section><section><h2>Where Things Go Wrong</h2><ul>{}</ul></section><section><h2>Suggestions</h2><ul>{}</ul></section><section><h2>On the Horizon</h2><ul>{}</ul></section><section><h2>Evidence</h2><ul>{}</ul></section></body></html>",
        report.metadata.sessions_scanned,
        report.metadata.sessions_analyzed,
        escape_html(&report.metadata.generated_report_path),
        escape_html(&report.at_a_glance.whats_working),
        escape_html(&report.at_a_glance.whats_hindering),
        escape_html(&report.at_a_glance.quick_wins),
        escape_html(&report.at_a_glance.ambitious_workflows),
        work_areas,
        escape_html(&report.interaction_style.narrative),
        what_works,
        friction,
        suggestions,
        horizon,
        evidence,
    )
}

fn default_insights_report_path() -> PathBuf {
    let home = dirs::home_dir().unwrap_or_else(|| PathBuf::from("."));
    home.join(".codex")
        .join("codex-threads")
        .join("insights")
        .join("report.html")
}

fn session_project_name(fact: &SessionFact) -> String {
    fact.thread
        .project_name
        .clone()
        .or_else(|| {
            fact.thread
                .cwd
                .as_deref()
                .and_then(|cwd| Path::new(cwd).file_name())
                .and_then(|name| name.to_str())
                .map(ToOwned::to_owned)
        })
        .unwrap_or_else(|| "unknown".to_owned())
}

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
        ("session_meta", _) => event
            .payload
            .get("cwd")
            .and_then(Value::as_str)
            .map(|cwd| EventSummaryItem {
                timestamp: event.timestamp.clone(),
                category: "session".to_owned(),
                detail: format!("started in {cwd}"),
            }),
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
    let exit_code = payload.get("exit_code").and_then(Value::as_i64).unwrap_or(0);
    let duration = format_duration(payload).unwrap_or_else(|| "unknown duration".to_owned());
    let mut detail = if exit_code == 0 {
        format!("`{}` completed successfully in {}", one_line(&command, 120), duration)
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
        let arguments = payload.get("arguments").and_then(Value::as_str).unwrap_or("");
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
        "ok"
            | "okay"
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

    for action in digest.key_actions.iter().rev().take(3).collect::<Vec<_>>().into_iter().rev() {
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
            "Add a recent-threads batch mode that generates one insight note per thread.".to_owned(),
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
    if insights.iter().any(|item| item.contains("interpretation layer")) {
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
        && command.get(1).and_then(Value::as_str).is_some_and(|part| part == "-lc")
    {
        return command.get(2).and_then(Value::as_str).map(ToOwned::to_owned);
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

    let snippet = slice_chars(&compact, start_char, end_char).trim().to_owned();
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
        assert_eq!(matches[0].snippet, "build a CLI that searches Codex threads");

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
                "我们测试一下这个cli，你利用这个cli读取我们最近的聊天看看有哪些可以改进的".to_owned(),
            ],
            key_actions: vec![EventSummaryItem {
                timestamp: Some("2026-04-13T05:00:06Z".to_owned()),
                category: "command".to_owned(),
                detail: "`messages search` completed successfully in 585ms".to_owned(),
            }],
            outcome: Some(
                "现在效果明显比上一版对了，messages search 已经改成 snippet-first。"
                    .to_owned(),
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
                started_at: Some(format!("2026-04-13T05:00:0{}Z", thread_id.chars().last().unwrap_or('0'))),
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
        let facts = vec![
            sample_fact("1", "mnemo", "mnemo one"),
            sample_fact("2", "mnemo", "mnemo two"),
            sample_fact("3", "opensource", "open one"),
            sample_fact("4", "mnemo", "mnemo three"),
            sample_fact("5", "physedit", "phys one"),
        ];
        let args = InsightsArgs {
            limit: 4,
            project: None,
            output: None,
            event_limit: 120,
            max_message_chars: 1600,
        };

        let selected = select_insight_facts(facts, &args);
        let projects = selected
            .iter()
            .map(session_project_name)
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
        let evidence = build_global_evidence(&[sample_fact("1", "mnemo", "plan mnemo")]);

        assert!(evidence.iter().any(|item| item.starts_with("User request:")));
        assert!(evidence.iter().any(|item| item.starts_with("Session objective:")));
    }
}
