use std::collections::{HashMap, HashSet};
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
    Read(ReadThreadArgs),
}

#[derive(Debug, Subcommand)]
enum EventsCommand {
    Read(ReadEventsArgs),
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
    text: String,
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
}

#[derive(Debug, Clone, Serialize)]
struct ThreadMessage {
    timestamp: Option<String>,
    role: String,
    phase: Option<String>,
    text: String,
}

#[derive(Debug, Clone, Serialize)]
struct EventItem {
    timestamp: Option<String>,
    kind: String,
    event_type: Option<String>,
    summary: Option<String>,
    payload: Value,
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
        Command::Messages { command } => match command {
            MessagesCommand::Search(args) => {
                render_messages_search(search_messages(&conn, &args.query, args.limit)?)
            }
        },
        Command::Threads { command } => match command {
            ThreadsCommand::Resolve(args) => {
                render_threads_resolve(resolve_threads(&conn, &args.query, args.limit)?)
            }
            ThreadsCommand::Read(args) => {
                render_thread_read(read_thread(&conn, &args.session_id, args.limit)?)
            }
        },
        Command::Events { command } => match command {
            EventsCommand::Read(args) => {
                render_events_read(read_events(&conn, &args.session_id, args.limit)?)
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
        writeln!(&mut text, "   {}", one_line(&item.text, 240))?;
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
    let mut stmt = conn.prepare(
        r#"
        SELECT
            m.thread_id,
            m.timestamp,
            m.role,
            m.phase,
            m.text,
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
        Ok(SearchMessageResult {
            thread_id: row.get(0)?,
            timestamp: row.get(1)?,
            role: row.get(2)?,
            phase: row.get(3)?,
            text: row.get(4)?,
            cwd: row.get(5)?,
            path: row.get(6)?,
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
            CASE
                WHEN instr(lower(t.thread_id), ?1) > 0
                  OR instr(lower(COALESCE(t.summary, '')), ?1) > 0
                  OR instr(lower(COALESCE(t.cwd, '')), ?1) > 0
                  OR instr(lower(COALESCE(t.path, '')), ?1) > 0
                THEN 1 ELSE 0
            END AS thread_match
        FROM threads t
        LEFT JOIN messages m ON m.thread_id = t.thread_id
        GROUP BY t.thread_id
        HAVING thread_match = 1 OR match_count > 0
        ORDER BY thread_match DESC, match_count DESC, COALESCE(last_match_at, t.started_at) DESC
        LIMIT ?2
        "#,
    )?;

    let rows = stmt.query_map(params![normalized, limit as i64], |row| {
        Ok(ThreadResolveResult {
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
        })
    })?;

    rows.collect::<rusqlite::Result<Vec<_>>>()
        .map_err(Into::into)
}

fn read_thread(
    conn: &Connection,
    session_id: &str,
    limit: Option<usize>,
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
            })
        })?;
        rows.collect::<rusqlite::Result<Vec<_>>>()?
    };

    Ok((thread, messages))
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

fn normalize_query(query: &str) -> String {
    query.trim().to_lowercase()
}

fn one_line(text: &str, max_len: usize) -> String {
    let normalized = text.split_whitespace().collect::<Vec<_>>().join(" ");
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

        let threads = resolve_threads(&conn, "example-project", 10).unwrap();
        assert_eq!(threads.len(), 1);

        let (thread, messages) =
            read_thread(&conn, "019d8510-9b67-7ff0-914c-cc313085e394", None).unwrap();
        assert_eq!(thread.message_count, 2);
        assert_eq!(messages.len(), 2);
    }
}
