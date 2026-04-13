const EXAMPLE_SESSION_SAMPLE_CAP: usize = 8;

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
    modes: Vec<String>,
    themes: Vec<String>,
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
struct EvidenceItem {
    id: String,
    thread_id: String,
    title: String,
    detail: String,
    project: String,
    mode: String,
    themes: Vec<String>,
    confidence: String,
}

#[derive(Debug, Clone, Serialize)]
struct LabeledCount {
    label: String,
    count: usize,
}

#[derive(Debug, Clone, Serialize)]
struct SignalCluster {
    label: String,
    count: usize,
    examples: Vec<String>,
    evidence_ids: Vec<String>,
    confidence: String,
}

#[derive(Debug, Clone, Serialize)]
struct AggregatedProject {
    name: String,
    session_count: usize,
    dominant_modes: Vec<String>,
    top_themes: Vec<String>,
    examples: Vec<String>,
}

#[derive(Debug, Clone, Serialize)]
struct ExampleSession {
    thread_id: String,
    project: String,
    mode: String,
    mode_confidence: String,
    themes: Vec<String>,
    objective: Option<String>,
    outcome: Option<String>,
    outcome_strength: String,
    outcome_confidence: String,
    evidence_ids: Vec<String>,
    classification_notes: Vec<String>,
}

#[derive(Debug, Clone, Serialize)]
struct BriefingSummary {
    sessions_scanned: usize,
    sessions_analyzed: usize,
    example_sessions_are_samples: bool,
    example_session_count: usize,
    example_session_cap: usize,
    project_filter: Option<String>,
    dominant_project: Option<String>,
    dominant_mode: Option<String>,
    validated_sessions: usize,
    iterative_sessions: usize,
    sessions_with_failures: usize,
    sessions_with_context_overload: usize,
    top_themes: Vec<String>,
}

#[derive(Debug, Clone, Serialize)]
struct BriefingPatterns {
    active_projects: Vec<AggregatedProject>,
    dominant_modes: Vec<LabeledCount>,
    recurring_themes: Vec<LabeledCount>,
    recurring_success_patterns: Vec<SignalCluster>,
    recurring_frictions: Vec<SignalCluster>,
    tool_usage: Vec<LabeledCount>,
    context_styles: Vec<LabeledCount>,
    outcome_strengths: Vec<LabeledCount>,
}

#[derive(Debug, Clone, Serialize)]
struct InsightsBriefing {
    schema_version: String,
    purpose: String,
    summary: BriefingSummary,
    patterns: BriefingPatterns,
    evidence: Vec<EvidenceItem>,
    example_sessions: Vec<ExampleSession>,
    uncertainties: Vec<String>,
    modeling_notes: Vec<String>,
}

#[derive(Debug, Clone, Serialize)]
struct SourceContract {
    canonical_sources: Vec<String>,
    noncanonical_sources: Vec<String>,
    consumption_order: Vec<String>,
}

#[derive(Debug, Clone, Serialize)]
struct CanonicalReceipt {
    sessions_scanned: usize,
    sessions_analyzed: usize,
    example_sessions_are_samples: bool,
    example_session_count: usize,
    example_session_cap: usize,
    dominant_project: Option<String>,
    dominant_mode: Option<String>,
    recurring_theme_labels: Vec<String>,
    evidence_ids: Vec<String>,
    recurring_friction_labels: Vec<String>,
    recurring_success_labels: Vec<String>,
}

#[derive(Debug, Clone, Serialize)]
struct PatternTraceItem {
    label: String,
    count: usize,
    confidence: String,
    evidence_ids: Vec<String>,
}

#[derive(Debug, Clone, Serialize)]
struct ProjectTraceItem {
    name: String,
    session_count: usize,
    dominant_modes: Vec<String>,
    top_themes: Vec<String>,
}

#[derive(Debug, Clone, Serialize)]
struct EvidenceTraceItem {
    id: String,
    thread_id: String,
    project: String,
    mode: String,
    confidence: String,
}

#[derive(Debug, Clone, Serialize)]
struct InsightsTrace {
    canonical_receipt: CanonicalReceipt,
    project_clusters: Vec<ProjectTraceItem>,
    recurring_success_patterns: Vec<PatternTraceItem>,
    recurring_frictions: Vec<PatternTraceItem>,
    evidence_receipt: Vec<EvidenceTraceItem>,
}

#[derive(Debug, Clone, Serialize)]
struct HeuristicDraft {
    at_a_glance: AtAGlance,
    work_areas: Vec<WorkArea>,
    interaction_style: InteractionStyle,
    what_works: Vec<InsightCard>,
    friction: Vec<FrictionCard>,
    suggestions: Vec<SuggestionCard>,
    on_the_horizon: Vec<String>,
    evidence: Vec<EvidenceItem>,
    content: String,
}

#[derive(Debug, Clone, Serialize)]
struct AggregatedInsightsData {
    analyzed_session_count: usize,
    active_projects: Vec<AggregatedProject>,
    dominant_modes: Vec<LabeledCount>,
    recurring_themes: Vec<LabeledCount>,
    recurring_frictions: Vec<SignalCluster>,
    recurring_success_patterns: Vec<SignalCluster>,
    tool_usage: Vec<LabeledCount>,
    context_styles: Vec<LabeledCount>,
    outcome_strengths: Vec<LabeledCount>,
    validated_sessions: usize,
    iterative_sessions: usize,
    sessions_with_outcomes: usize,
    sessions_with_failures: usize,
    sessions_with_context_overload: usize,
    example_sessions: Vec<ExampleSession>,
}

#[derive(Debug, Clone, Serialize)]
struct InsightsReport {
    metadata: InsightsMetadata,
    aggregated: AggregatedInsightsData,
    briefing: InsightsBriefing,
    source_contract: SourceContract,
    trace: InsightsTrace,
    heuristic_draft: HeuristicDraft,
    #[serde(skip_serializing)]
    at_a_glance: AtAGlance,
    #[serde(skip_serializing)]
    work_areas: Vec<WorkArea>,
    #[serde(skip_serializing)]
    interaction_style: InteractionStyle,
    #[serde(skip_serializing)]
    what_works: Vec<InsightCard>,
    #[serde(skip_serializing)]
    friction: Vec<FrictionCard>,
    #[serde(skip_serializing)]
    suggestions: Vec<SuggestionCard>,
    #[serde(skip_serializing)]
    on_the_horizon: Vec<String>,
    #[serde(skip_serializing)]
    evidence: Vec<EvidenceItem>,
    #[serde(skip_serializing)]
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

#[derive(Debug, Clone, Serialize, PartialEq, Eq, Hash, PartialOrd, Ord)]
#[serde(rename_all = "snake_case")]
enum SessionMode {
    Research,
    Planning,
    Implementation,
    Debugging,
    Review,
    Exploratory,
}

impl SessionMode {
    fn label(&self) -> &'static str {
        match self {
            SessionMode::Research => "Research",
            SessionMode::Planning => "Planning",
            SessionMode::Implementation => "Implementation",
            SessionMode::Debugging => "Debugging",
            SessionMode::Review => "Review",
            SessionMode::Exploratory => "Exploratory",
        }
    }
}

#[derive(Debug, Clone, Serialize, PartialEq, Eq, Hash, PartialOrd, Ord)]
#[serde(rename_all = "snake_case")]
enum ContextStyle {
    Lean,
    Blended,
    ContextHeavy,
}

impl ContextStyle {
    fn label(&self) -> &'static str {
        match self {
            ContextStyle::Lean => "Lean context",
            ContextStyle::Blended => "Blended context",
            ContextStyle::ContextHeavy => "Context-heavy",
        }
    }
}

#[derive(Debug, Clone, Serialize, PartialEq, Eq, Hash, PartialOrd, Ord)]
#[serde(rename_all = "snake_case")]
enum OutcomeStrength {
    Weak,
    Partial,
    Strong,
}

impl OutcomeStrength {
    fn label(&self) -> &'static str {
        match self {
            OutcomeStrength::Weak => "Weak outcome",
            OutcomeStrength::Partial => "Partial outcome",
            OutcomeStrength::Strong => "Strong outcome",
        }
    }
}

#[derive(Debug, Clone, Serialize)]
struct SessionFacets {
    session_id: String,
    project: String,
    primary_mode: SessionMode,
    mode_confidence: String,
    themes: Vec<String>,
    tools_used: Vec<String>,
    success_signals: Vec<String>,
    friction_signals: Vec<String>,
    context_style: ContextStyle,
    outcome_strength: OutcomeStrength,
    outcome_confidence: String,
    classification_notes: Vec<String>,
}

#[derive(Debug, Clone)]
struct SessionAnalysis {
    fact: SessionFact,
    facets: SessionFacets,
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

fn render_events_summary(
    payload: (ThreadSummary, Vec<EventSummaryItem>),
) -> Result<RenderedOutput> {
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
        .or_else(|| {
            thread
                .summary
                .as_ref()
                .map(|summary| one_line(summary, 220))
        });

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
            messages.iter().rev().find(|message| {
                message.role == "assistant" && is_substantive_message(&message.text)
            })
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
    let mut analyses = Vec::new();

    for thread in candidate_threads {
        if let Some(analysis) =
            build_session_analysis(conn, thread, args.event_limit, args.max_message_chars)?
        {
            analyses.push(analysis);
        }
    }

    analyses = select_insight_analyses(analyses, args);

    if analyses.is_empty() {
        bail!("no analyzable sessions found for insights");
    }

    let report_path = args
        .output
        .clone()
        .unwrap_or_else(default_insights_report_path);
    let report = build_global_insights_report(
        &analyses,
        sessions_scanned,
        args.project.clone(),
        &report_path,
    );
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
    let command_failures = events
        .iter()
        .filter(|event| event.category == "error")
        .count();

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

fn build_session_analysis(
    conn: &Connection,
    thread: ThreadSummary,
    event_limit: usize,
    max_message_chars: usize,
) -> Result<Option<SessionAnalysis>> {
    let Some(fact) = build_session_fact(conn, thread, event_limit, max_message_chars)? else {
        return Ok(None);
    };
    let facets = build_session_facets(&fact);
    Ok(Some(SessionAnalysis { fact, facets }))
}

fn select_insight_analyses(
    mut analyses: Vec<SessionAnalysis>,
    args: &InsightsArgs,
) -> Vec<SessionAnalysis> {
    if analyses.len() <= args.limit {
        return analyses;
    }

    if args.project.is_some() {
        analyses.truncate(args.limit);
        return analyses;
    }

    let mut buckets: HashMap<String, VecDeque<SessionAnalysis>> = HashMap::new();
    let mut order = Vec::new();
    for analysis in analyses {
        let fact = &analysis.fact;
        let project = session_project_name(&fact);
        if !buckets.contains_key(&project) {
            order.push(project.clone());
        }
        buckets.entry(project).or_default().push_back(analysis);
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
    analyses: &[SessionAnalysis],
    sessions_scanned: usize,
    project_filter: Option<String>,
    report_path: &Path,
) -> InsightsReport {
    let evidence = build_global_evidence(analyses);
    let evidence_index = build_evidence_index(&evidence);
    let aggregated = aggregate_session_facets(analyses, &evidence_index);
    let work_areas = build_work_areas(&aggregated);
    let interaction_style = build_interaction_style(&aggregated, &work_areas);
    let what_works = build_what_works(&aggregated, &work_areas);
    let friction = build_friction_cards(&aggregated);
    let suggestions = build_suggestion_cards(&aggregated, &work_areas, &friction);
    let on_the_horizon = build_horizon_items(&aggregated, &work_areas);
    let at_a_glance = build_at_a_glance(
        &aggregated,
        &work_areas,
        &interaction_style,
        &friction,
        &suggestions,
    );
    let metadata = InsightsMetadata {
        sessions_scanned,
        sessions_analyzed: analyses.len(),
        project_filter,
        generated_report_path: report_path.display().to_string(),
    };
    let briefing = build_insights_briefing(
        &metadata,
        &aggregated,
        &evidence,
        &analyses
            .iter()
            .take(EXAMPLE_SESSION_SAMPLE_CAP)
            .map(|analysis| ExampleSession {
                thread_id: analysis.fact.thread.thread_id.clone(),
                project: analysis.facets.project.clone(),
                mode: analysis.facets.primary_mode.label().to_owned(),
                mode_confidence: analysis.facets.mode_confidence.clone(),
                themes: analysis.facets.themes.clone(),
                objective: analysis.fact.objective.clone(),
                outcome: analysis.fact.outcome.clone(),
                outcome_strength: analysis.facets.outcome_strength.label().to_owned(),
                outcome_confidence: analysis.facets.outcome_confidence.clone(),
                evidence_ids: evidence_index
                    .get(&analysis.fact.thread.thread_id)
                    .cloned()
                    .unwrap_or_default(),
                classification_notes: analysis.facets.classification_notes.clone(),
            })
            .collect::<Vec<_>>(),
    );
    let source_contract = build_source_contract();
    let trace = build_insights_trace(&metadata, &aggregated, &evidence);
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
    let heuristic_draft = HeuristicDraft {
        at_a_glance: at_a_glance.clone(),
        work_areas: work_areas.clone(),
        interaction_style: interaction_style.clone(),
        what_works: what_works.clone(),
        friction: friction.clone(),
        suggestions: suggestions.clone(),
        on_the_horizon: on_the_horizon.clone(),
        evidence: evidence.clone(),
        content: content.clone(),
    };

    InsightsReport {
        metadata,
        aggregated,
        briefing,
        source_contract,
        trace,
        heuristic_draft,
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

fn build_source_contract() -> SourceContract {
    SourceContract {
        canonical_sources: vec![
            "data.trace".to_owned(),
            "data.briefing.summary".to_owned(),
            "data.briefing.patterns".to_owned(),
            "data.briefing.evidence".to_owned(),
            "data.briefing.example_sessions".to_owned(),
            "data.briefing.uncertainties".to_owned(),
            "data.briefing.modeling_notes".to_owned(),
        ],
        noncanonical_sources: vec![
            "data.heuristic_draft".to_owned(),
            "data.heuristic_draft.content".to_owned(),
        ],
        consumption_order: vec![
            "Read `data.trace` first to lock the current payload receipt and valid evidence ids."
                .to_owned(),
            "Use `data.briefing` as the canonical fact layer for counts, patterns, evidence, and uncertainty."
                .to_owned(),
            "Use `data.heuristic_draft` only as a wording/reference draft after the canonical narrative is already formed."
                .to_owned(),
        ],
    }
}

fn build_insights_briefing(
    metadata: &InsightsMetadata,
    aggregated: &AggregatedInsightsData,
    evidence: &[EvidenceItem],
    example_sessions: &[ExampleSession],
) -> InsightsBriefing {
    let dominant_project = aggregated.active_projects.first().map(|item| item.name.clone());
    let dominant_mode = aggregated.dominant_modes.first().map(|item| item.label.clone());
    let top_themes = aggregated
        .recurring_themes
        .iter()
        .take(5)
        .map(|item| item.label.clone())
        .collect::<Vec<_>>();
    let uncertainties = build_briefing_uncertainties(metadata, aggregated);

    InsightsBriefing {
        schema_version: "insights-briefing-v1".to_owned(),
        purpose: "Structured backend payload for model-written Codex insights reports."
            .to_owned(),
        summary: BriefingSummary {
            sessions_scanned: metadata.sessions_scanned,
            sessions_analyzed: metadata.sessions_analyzed,
            example_sessions_are_samples: true,
            example_session_count: example_sessions.len(),
            example_session_cap: EXAMPLE_SESSION_SAMPLE_CAP,
            project_filter: metadata.project_filter.clone(),
            dominant_project,
            dominant_mode,
            validated_sessions: aggregated.validated_sessions,
            iterative_sessions: aggregated.iterative_sessions,
            sessions_with_failures: aggregated.sessions_with_failures,
            sessions_with_context_overload: aggregated.sessions_with_context_overload,
            top_themes,
        },
        patterns: BriefingPatterns {
            active_projects: aggregated.active_projects.clone(),
            dominant_modes: aggregated.dominant_modes.clone(),
            recurring_themes: aggregated.recurring_themes.clone(),
            recurring_success_patterns: aggregated.recurring_success_patterns.clone(),
            recurring_frictions: aggregated.recurring_frictions.clone(),
            tool_usage: aggregated.tool_usage.clone(),
            context_styles: aggregated.context_styles.clone(),
            outcome_strengths: aggregated.outcome_strengths.clone(),
        },
        evidence: evidence.to_vec(),
        example_sessions: example_sessions.to_vec(),
        uncertainties,
        modeling_notes: vec![
            "Use recurring patterns and evidence as the source of truth; treat the human-facing `content` field as a heuristic draft, not canonical analysis.".to_owned(),
            "Prefer project/mode/theme clusters over raw tool frequency when writing narrative sections.".to_owned(),
            "Call out uncertainty explicitly when low-execution-evidence or context-overload signals are high.".to_owned(),
        ],
    }
}

fn build_evidence_index(evidence: &[EvidenceItem]) -> HashMap<String, Vec<String>> {
    let mut index = HashMap::new();
    for item in evidence {
        index
            .entry(item.thread_id.clone())
            .or_insert_with(Vec::new)
            .push(item.id.clone());
    }
    index
}

fn build_insights_trace(
    metadata: &InsightsMetadata,
    aggregated: &AggregatedInsightsData,
    evidence: &[EvidenceItem],
) -> InsightsTrace {
    InsightsTrace {
        canonical_receipt: CanonicalReceipt {
            sessions_scanned: metadata.sessions_scanned,
            sessions_analyzed: metadata.sessions_analyzed,
            example_sessions_are_samples: true,
            example_session_count: aggregated.example_sessions.len(),
            example_session_cap: EXAMPLE_SESSION_SAMPLE_CAP,
            dominant_project: aggregated.active_projects.first().map(|item| item.name.clone()),
            dominant_mode: aggregated.dominant_modes.first().map(|item| item.label.clone()),
            recurring_theme_labels: aggregated
                .recurring_themes
                .iter()
                .map(|item| item.label.clone())
                .collect(),
            evidence_ids: evidence.iter().map(|item| item.id.clone()).collect(),
            recurring_friction_labels: aggregated
                .recurring_frictions
                .iter()
                .map(|item| item.label.clone())
                .collect(),
            recurring_success_labels: aggregated
                .recurring_success_patterns
                .iter()
                .map(|item| item.label.clone())
                .collect(),
        },
        project_clusters: aggregated
            .active_projects
            .iter()
            .map(|item| ProjectTraceItem {
                name: item.name.clone(),
                session_count: item.session_count,
                dominant_modes: item.dominant_modes.clone(),
                top_themes: item.top_themes.clone(),
            })
            .collect(),
        recurring_success_patterns: aggregated
            .recurring_success_patterns
            .iter()
            .map(|item| PatternTraceItem {
                label: item.label.clone(),
                count: item.count,
                confidence: item.confidence.clone(),
                evidence_ids: item.evidence_ids.clone(),
            })
            .collect(),
        recurring_frictions: aggregated
            .recurring_frictions
            .iter()
            .map(|item| PatternTraceItem {
                label: item.label.clone(),
                count: item.count,
                confidence: item.confidence.clone(),
                evidence_ids: item.evidence_ids.clone(),
            })
            .collect(),
        evidence_receipt: evidence
            .iter()
            .map(|item| EvidenceTraceItem {
                id: item.id.clone(),
                thread_id: item.thread_id.clone(),
                project: item.project.clone(),
                mode: item.mode.clone(),
                confidence: item.confidence.clone(),
            })
            .collect(),
    }
}

fn build_briefing_uncertainties(
    metadata: &InsightsMetadata,
    aggregated: &AggregatedInsightsData,
) -> Vec<String> {
    let mut uncertainties = Vec::new();

    if aggregated.example_sessions.len() < metadata.sessions_analyzed {
        uncertainties.push(format!(
            "Example sessions are capped at {} items for display; use `sessions_analyzed` as the real denominator for narrative claims.",
            EXAMPLE_SESSION_SAMPLE_CAP
        ));
    }
    if aggregated.sessions_with_context_overload > 0 {
        uncertainties.push(
            "Some sessions begin with pasted handoffs or collapsed context, so objective/theme inference may reflect summary artifacts as well as direct user intent."
                .to_owned(),
        );
    }
    if aggregated
        .recurring_frictions
        .iter()
        .any(|item| item.label == "Low execution evidence")
    {
        uncertainties.push(
            "A share of analyzed sessions lacks direct command/tool evidence, so outcome strength is partly inferred from thread endings rather than explicit validation."
                .to_owned(),
        );
    }
    if aggregated.sessions_with_failures > 0 && aggregated.validated_sessions == 0 {
        uncertainties.push(
            "This sample shows failures without successful validations, which can overstate friction relative to the broader archive."
                .to_owned(),
        );
    }

    uncertainties
}

fn build_session_facets(fact: &SessionFact) -> SessionFacets {
    let primary_mode = classify_primary_mode(fact);
    let tools_used = extract_tools_used(fact);
    let outcome_strength = classify_outcome_strength(fact, &tools_used);
    let context_style = classify_context_style(fact);
    let themes = extract_themes(fact, &primary_mode);
    let mode_confidence = classify_mode_confidence(fact, &primary_mode);
    let outcome_confidence = classify_outcome_confidence(fact, &outcome_strength);
    let success_signals = extract_success_signals(fact, &tools_used, &outcome_strength);
    let friction_signals = extract_friction_signals(fact, &outcome_strength);
    let classification_notes = build_classification_notes(
        fact,
        &primary_mode,
        &mode_confidence,
        &themes,
        &outcome_strength,
        &outcome_confidence,
    );

    SessionFacets {
        session_id: fact.thread.thread_id.clone(),
        project: session_project_name(fact),
        primary_mode,
        mode_confidence,
        themes,
        tools_used,
        success_signals,
        friction_signals,
        context_style,
        outcome_strength,
        outcome_confidence,
        classification_notes,
    }
}

fn aggregate_session_facets(
    analyses: &[SessionAnalysis],
    evidence_index: &HashMap<String, Vec<String>>,
) -> AggregatedInsightsData {
    let mut project_groups: HashMap<String, Vec<&SessionAnalysis>> = HashMap::new();
    let mut mode_counts = HashMap::new();
    let mut theme_counts = HashMap::new();
    let mut tool_counts = HashMap::new();
    let mut context_counts = HashMap::new();
    let mut outcome_counts = HashMap::new();
    let mut friction_counts = HashMap::new();
    let mut success_counts = HashMap::new();
    let mut friction_examples: HashMap<String, Vec<String>> = HashMap::new();
    let mut success_examples: HashMap<String, Vec<String>> = HashMap::new();
    let mut friction_evidence_ids: HashMap<String, Vec<String>> = HashMap::new();
    let mut success_evidence_ids: HashMap<String, Vec<String>> = HashMap::new();
    let mut example_sessions = Vec::new();
    let mut validated_sessions = 0usize;
    let mut iterative_sessions = 0usize;
    let mut sessions_with_outcomes = 0usize;
    let mut sessions_with_failures = 0usize;
    let mut sessions_with_context_overload = 0usize;

    for analysis in analyses {
        let fact = &analysis.fact;
        let facets = &analysis.facets;
        let project = facets.project.clone();

        project_groups
            .entry(project.clone())
            .or_default()
            .push(analysis);
        increment_count(&mut mode_counts, facets.primary_mode.label());
        increment_count(&mut context_counts, facets.context_style.label());
        increment_count(&mut outcome_counts, facets.outcome_strength.label());

        for theme in &facets.themes {
            increment_count(&mut theme_counts, theme);
        }
        for tool in &facets.tools_used {
            increment_count(&mut tool_counts, tool);
        }
        for signal in &facets.friction_signals {
            increment_count(&mut friction_counts, signal);
            push_cluster_example(
                &mut friction_examples,
                signal,
                best_session_example(fact),
                3,
            );
            push_cluster_evidence_ids(
                &mut friction_evidence_ids,
                signal,
                evidence_index.get(&fact.thread.thread_id),
                4,
            );
        }
        for signal in &facets.success_signals {
            increment_count(&mut success_counts, signal);
            push_cluster_example(&mut success_examples, signal, best_session_example(fact), 3);
            push_cluster_evidence_ids(
                &mut success_evidence_ids,
                signal,
                evidence_index.get(&fact.thread.thread_id),
                4,
            );
        }

        if fact.command_successes > 0 {
            validated_sessions += 1;
        }
        if fact.substantive_user_messages >= 2 {
            iterative_sessions += 1;
        }
        if fact.outcome.is_some() {
            sessions_with_outcomes += 1;
        }
        if fact.command_failures > 0 {
            sessions_with_failures += 1;
        }
        if fact.cleaned_context_messages > 0 {
            sessions_with_context_overload += 1;
        }

        if example_sessions.len() < EXAMPLE_SESSION_SAMPLE_CAP {
            example_sessions.push(ExampleSession {
                thread_id: fact.thread.thread_id.clone(),
                project,
                mode: facets.primary_mode.label().to_owned(),
                mode_confidence: facets.mode_confidence.clone(),
                themes: facets.themes.clone(),
                objective: fact.objective.clone(),
                outcome: fact.outcome.clone(),
                outcome_strength: facets.outcome_strength.label().to_owned(),
                outcome_confidence: facets.outcome_confidence.clone(),
                evidence_ids: evidence_index
                    .get(&fact.thread.thread_id)
                    .cloned()
                    .unwrap_or_default(),
                classification_notes: facets.classification_notes.clone(),
            });
        }
    }

    let mut active_projects = project_groups
        .into_iter()
        .map(|(name, group)| {
            let mut project_mode_counts = HashMap::new();
            let mut project_theme_counts = HashMap::new();
            let mut examples = Vec::new();

            for analysis in group.iter() {
                increment_count(
                    &mut project_mode_counts,
                    analysis.facets.primary_mode.label(),
                );
                for theme in &analysis.facets.themes {
                    increment_count(&mut project_theme_counts, theme);
                }
                push_unique(&mut examples, best_session_example(&analysis.fact), 2);
            }

            AggregatedProject {
                name,
                session_count: group.len(),
                dominant_modes: sorted_count_labels(&project_mode_counts, 2),
                top_themes: sorted_count_labels(&project_theme_counts, 3),
                examples,
            }
        })
        .collect::<Vec<_>>();
    active_projects.sort_by(|left, right| {
        right
            .session_count
            .cmp(&left.session_count)
            .then_with(|| left.name.cmp(&right.name))
    });

    AggregatedInsightsData {
        analyzed_session_count: analyses.len(),
        active_projects,
        dominant_modes: sorted_counts(&mode_counts, 6),
        recurring_themes: sorted_counts(&theme_counts, 8),
        recurring_frictions: sorted_clusters(
            &friction_counts,
            &friction_examples,
            &friction_evidence_ids,
            4,
        ),
        recurring_success_patterns: sorted_clusters(
            &success_counts,
            &success_examples,
            &success_evidence_ids,
            4,
        ),
        tool_usage: sorted_counts(&tool_counts, 8),
        context_styles: sorted_counts(&context_counts, 3),
        outcome_strengths: sorted_counts(&outcome_counts, 3),
        validated_sessions,
        iterative_sessions,
        sessions_with_outcomes,
        sessions_with_failures,
        sessions_with_context_overload,
        example_sessions,
    }
}

fn classify_primary_mode(fact: &SessionFact) -> SessionMode {
    let text = session_text_blob(fact);
    let has_errors = fact.command_failures > 0
        || fact
            .key_actions
            .iter()
            .any(|action| action.category == "error");
    let has_delivery = fact.command_successes > 0
        || fact
            .key_actions
            .iter()
            .any(|action| matches!(action.category.as_str(), "tool" | "command"));

    let mut scores: HashMap<SessionMode, usize> = HashMap::new();
    add_keyword_score(
        &mut scores,
        &text,
        SessionMode::Planning,
        &[
            "plan",
            "planning",
            "handoff",
            "design",
            "spec",
            "roadmap",
            "break down",
            "architecture",
        ],
        2,
    );
    add_keyword_score(
        &mut scores,
        &text,
        SessionMode::Research,
        &[
            "research",
            "analyze",
            "analysis",
            "understand",
            "read",
            "summarize",
            "reference",
            "compare",
            "insight",
        ],
        2,
    );
    add_keyword_score(
        &mut scores,
        &text,
        SessionMode::Implementation,
        &[
            "implement",
            "build",
            "create",
            "add",
            "write",
            "update",
            "refactor",
            "feature",
            "ship",
        ],
        2,
    );
    add_keyword_score(
        &mut scores,
        &text,
        SessionMode::Debugging,
        &[
            "debug",
            "fix",
            "failing",
            "failure",
            "error",
            "broken",
            "bug",
            "regression",
            "panic",
            "exception",
        ],
        2,
    );
    add_keyword_score(
        &mut scores,
        &text,
        SessionMode::Review,
        &[
            "review",
            "audit",
            "critique",
            "regression risk",
            "findings",
            "pull request",
            "pr",
            "diff",
        ],
        2,
    );
    add_keyword_score(
        &mut scores,
        &text,
        SessionMode::Exploratory,
        &[
            "explore",
            "exploratory",
            "brainstorm",
            "idea",
            "investigate",
        ],
        1,
    );

    if has_errors {
        *scores.entry(SessionMode::Debugging).or_default() += 3;
    }
    if has_delivery {
        *scores.entry(SessionMode::Implementation).or_default() += 2;
    }
    if fact.substantive_user_messages >= 2 {
        *scores.entry(SessionMode::Implementation).or_default() += 1;
        *scores.entry(SessionMode::Planning).or_default() += 1;
    }

    let priority = [
        SessionMode::Debugging,
        SessionMode::Implementation,
        SessionMode::Planning,
        SessionMode::Review,
        SessionMode::Research,
        SessionMode::Exploratory,
    ];

    priority
        .iter()
        .filter_map(|mode| scores.get(mode).map(|score| (mode, *score)))
        .max_by(|left, right| left.1.cmp(&right.1))
        .map(|(mode, score)| {
            if score == 0 && !has_delivery && !has_errors {
                SessionMode::Exploratory
            } else {
                mode.clone()
            }
        })
        .unwrap_or_else(|| {
            if has_delivery {
                SessionMode::Implementation
            } else if has_errors {
                SessionMode::Debugging
            } else {
                SessionMode::Exploratory
            }
        })
}

fn extract_themes(fact: &SessionFact, primary_mode: &SessionMode) -> Vec<String> {
    let text = session_text_blob(fact);
    let mut themes = Vec::new();
    let theme_map = [
        ("CLI workflows", vec!["cli", "command", "shell", "terminal"]),
        (
            "Search & retrieval",
            vec![
                "search", "query", "index", "resolve", "thread", "session", "archive",
            ],
        ),
        (
            "Reporting & insights",
            vec![
                "report",
                "insight",
                "summary",
                "summarize",
                "html",
                "dashboard",
            ],
        ),
        (
            "Documentation & planning",
            vec![
                "docs",
                "handoff",
                "plan",
                "design",
                "spec",
                "readme",
                "architecture",
            ],
        ),
        (
            "Testing & validation",
            vec![
                "test",
                "assert",
                "verify",
                "validation",
                "regression",
                "cargo test",
            ],
        ),
        (
            "Rust implementation",
            vec!["rust", "cargo", ".rs", "src/main.rs"],
        ),
        (
            "TypeScript tooling",
            vec!["typescript", "javascript", "node", "npm", "pnpm", ".ts"],
        ),
        (
            "Frontend & HTML",
            vec!["frontend", "html", "css", "ui", "page", "layout"],
        ),
        (
            "Git & release",
            vec![
                "git",
                "commit",
                "branch",
                "pull request",
                "release",
                "merge",
            ],
        ),
        (
            "Agents & orchestration",
            vec!["agent", "subagent", "swarm", "parallel", "team"],
        ),
        (
            "Debugging & reliability",
            vec![
                "debug",
                "error",
                "failure",
                "bug",
                "panic",
                "exception",
                "timeout",
            ],
        ),
        (
            "Data & indexing",
            vec!["sqlite", "database", "index", "parser", "sync", "jsonl"],
        ),
    ];

    for (label, needles) in theme_map {
        if needles.iter().any(|needle| text.contains(needle)) {
            push_unique(&mut themes, label.to_owned(), 4);
        }
    }

    if themes.is_empty() {
        let fallback = match primary_mode {
            SessionMode::Research => "Exploration & analysis",
            SessionMode::Planning => "Planning & design",
            SessionMode::Implementation => "Implementation & delivery",
            SessionMode::Debugging => "Debugging & reliability",
            SessionMode::Review => "Review & QA",
            SessionMode::Exploratory => "Exploratory work",
        };
        themes.push(fallback.to_owned());
    }

    themes
}

fn extract_tools_used(fact: &SessionFact) -> Vec<String> {
    let mut tools = Vec::new();

    for action in &fact.key_actions {
        match action.category.as_str() {
            "tool" => {
                let name = action
                    .detail
                    .split_whitespace()
                    .next()
                    .unwrap_or_default()
                    .trim_end_matches("()");
                if !name.is_empty() {
                    push_unique(&mut tools, canonical_tool_name(name), 6);
                }
            }
            "command" | "error" => {
                if let Some(command) = extract_command_name(&action.detail) {
                    push_unique(&mut tools, command, 6);
                }
            }
            _ => {}
        }
    }

    if tools.is_empty() && (fact.command_successes > 0 || fact.command_failures > 0) {
        tools.push("exec_command".to_owned());
    }

    tools
}

fn extract_success_signals(
    fact: &SessionFact,
    tools_used: &[String],
    outcome_strength: &OutcomeStrength,
) -> Vec<String> {
    let mut signals = Vec::new();

    if fact.command_successes > 0 {
        push_unique(&mut signals, "Validated with real commands".to_owned(), 4);
    }
    if fact.substantive_user_messages >= 2 {
        push_unique(&mut signals, "Iterative refinement".to_owned(), 4);
    }
    if matches!(outcome_strength, OutcomeStrength::Strong) {
        push_unique(&mut signals, "Clear thread outcomes".to_owned(), 4);
    }
    if tools_used.iter().any(|tool| tool == "apply_patch") {
        push_unique(&mut signals, "Direct code edits".to_owned(), 4);
    }

    signals
}

fn extract_friction_signals(fact: &SessionFact, outcome_strength: &OutcomeStrength) -> Vec<String> {
    let mut signals = Vec::new();

    if fact.command_failures > 0 {
        push_unique(&mut signals, "Execution friction".to_owned(), 4);
    }
    if fact.cleaned_context_messages > 0 {
        push_unique(&mut signals, "Context overload".to_owned(), 4);
    }
    if matches!(outcome_strength, OutcomeStrength::Weak) {
        push_unique(&mut signals, "Soft thread endings".to_owned(), 4);
    }
    if fact.key_actions.is_empty() && fact.command_successes == 0 && fact.command_failures == 0 {
        push_unique(&mut signals, "Low execution evidence".to_owned(), 4);
    }

    signals
}

fn classify_context_style(fact: &SessionFact) -> ContextStyle {
    if fact.cleaned_context_messages > 0 && fact.substantive_user_messages <= 1 {
        ContextStyle::ContextHeavy
    } else if fact.cleaned_context_messages > 0 {
        ContextStyle::Blended
    } else {
        ContextStyle::Lean
    }
}

fn classify_outcome_strength(fact: &SessionFact, tools_used: &[String]) -> OutcomeStrength {
    if fact.outcome.is_some()
        && fact.command_failures == 0
        && (fact.command_successes > 0
            || tools_used.iter().any(|tool| tool == "apply_patch")
            || fact.key_actions.len() >= 2)
    {
        OutcomeStrength::Strong
    } else if fact.outcome.is_some() || fact.command_successes > 0 {
        OutcomeStrength::Partial
    } else {
        OutcomeStrength::Weak
    }
}

fn classify_mode_confidence(fact: &SessionFact, primary_mode: &SessionMode) -> String {
    let text = session_text_blob(fact);
    let matched = mode_keywords(primary_mode)
        .iter()
        .filter(|needle| text.contains(**needle))
        .count();
    let confidence = if matched >= 2
        || (matches!(primary_mode, SessionMode::Implementation) && fact.command_successes > 0)
        || (matches!(primary_mode, SessionMode::Debugging) && fact.command_failures > 0)
    {
        "high"
    } else if matched >= 1 || fact.substantive_user_messages >= 2 {
        "medium"
    } else {
        "low"
    };

    confidence.to_owned()
}

fn classify_outcome_confidence(
    fact: &SessionFact,
    outcome_strength: &OutcomeStrength,
) -> String {
    let confidence = match outcome_strength {
        OutcomeStrength::Strong if fact.command_successes > 0 && fact.command_failures == 0 => {
            "high"
        }
        OutcomeStrength::Partial if fact.outcome.is_some() || fact.command_successes > 0 => "medium",
        OutcomeStrength::Weak if fact.outcome.is_none() && fact.command_successes == 0 => "medium",
        _ => "low",
    };

    confidence.to_owned()
}

fn build_classification_notes(
    fact: &SessionFact,
    primary_mode: &SessionMode,
    mode_confidence: &str,
    themes: &[String],
    outcome_strength: &OutcomeStrength,
    outcome_confidence: &str,
) -> Vec<String> {
    let text = session_text_blob(fact);
    let mut notes = Vec::new();
    let matched_mode_keywords = mode_keywords(primary_mode)
        .iter()
        .filter(|needle| text.contains(**needle))
        .copied()
        .collect::<Vec<_>>();
    if matched_mode_keywords.is_empty() {
        notes.push(format!(
            "Mode classified as {} with {} confidence based on execution shape rather than explicit keywords.",
            primary_mode.label(),
            mode_confidence
        ));
    } else {
        notes.push(format!(
            "Mode classified as {} with {} confidence because the session mentions {}.",
            primary_mode.label(),
            mode_confidence,
            matched_mode_keywords.join(", ")
        ));
    }

    if !themes.is_empty() {
        notes.push(format!(
            "Themes selected from repeated objective/request terms and execution evidence: {}.",
            themes.join(", ")
        ));
    }

    let outcome_basis = match outcome_strength {
        OutcomeStrength::Strong => {
            "detected outcome text plus successful validation without observed failures"
        }
        OutcomeStrength::Partial => "detected outcome text or partial validation evidence",
        OutcomeStrength::Weak => "lack of explicit outcome or direct validation evidence",
    };
    notes.push(format!(
        "Outcome classified as {} with {} confidence based on {}.",
        outcome_strength.label(),
        outcome_confidence,
        outcome_basis
    ));

    if fact.cleaned_context_messages > 0 {
        notes.push(
            "Collapsed or pasted context was present, so objective/theme inference may be influenced by summary artifacts."
                .to_owned(),
        );
    }

    notes
}

fn mode_keywords(mode: &SessionMode) -> &'static [&'static str] {
    match mode {
        SessionMode::Research => &[
            "research",
            "analyze",
            "analysis",
            "understand",
            "read",
            "summarize",
            "reference",
            "compare",
            "insight",
        ],
        SessionMode::Planning => &[
            "plan",
            "planning",
            "handoff",
            "design",
            "spec",
            "roadmap",
            "break down",
            "architecture",
        ],
        SessionMode::Implementation => &[
            "implement",
            "build",
            "create",
            "add",
            "write",
            "update",
            "refactor",
            "feature",
            "ship",
        ],
        SessionMode::Debugging => &[
            "debug",
            "fix",
            "failing",
            "failure",
            "error",
            "broken",
            "bug",
            "regression",
            "panic",
            "exception",
        ],
        SessionMode::Review => &[
            "review",
            "audit",
            "critique",
            "regression risk",
            "findings",
            "pull request",
            "pr",
            "diff",
        ],
        SessionMode::Exploratory => &[
            "explore",
            "exploratory",
            "brainstorm",
            "idea",
            "investigate",
        ],
    }
}

fn session_text_blob(fact: &SessionFact) -> String {
    [
        fact.thread.summary.as_deref(),
        fact.objective.as_deref(),
        fact.outcome.as_deref(),
    ]
    .into_iter()
    .flatten()
    .map(normalize_query)
    .chain(fact.key_requests.iter().map(|item| normalize_query(item)))
    .chain(
        fact.key_actions
            .iter()
            .map(|item| normalize_query(&item.detail)),
    )
    .collect::<Vec<_>>()
    .join(" ")
}

fn best_session_example(fact: &SessionFact) -> String {
    fact.objective
        .as_ref()
        .or(fact.thread.summary.as_ref())
        .or(fact.outcome.as_ref())
        .map(|value| one_line(value, 140))
        .unwrap_or_else(|| format!("session {}", &fact.thread.thread_id[..8]))
}

fn increment_count<S: AsRef<str>>(counts: &mut HashMap<String, usize>, label: S) {
    *counts.entry(label.as_ref().to_owned()).or_default() += 1;
}

fn push_cluster_example(
    examples: &mut HashMap<String, Vec<String>>,
    label: &str,
    example: String,
    limit: usize,
) {
    let bucket = examples.entry(label.to_owned()).or_default();
    if !bucket
        .iter()
        .any(|existing| normalize_summary_value(existing) == normalize_summary_value(&example))
        && bucket.len() < limit
    {
        bucket.push(example);
    }
}

fn push_cluster_evidence_ids(
    clusters: &mut HashMap<String, Vec<String>>,
    label: &str,
    evidence_ids: Option<&Vec<String>>,
    limit: usize,
) {
    let Some(evidence_ids) = evidence_ids else {
        return;
    };
    let bucket = clusters.entry(label.to_owned()).or_default();
    for evidence_id in evidence_ids {
        if bucket.len() >= limit {
            break;
        }
        if !bucket.contains(evidence_id) {
            bucket.push(evidence_id.clone());
        }
    }
}

fn sorted_counts(counts: &HashMap<String, usize>, limit: usize) -> Vec<LabeledCount> {
    let mut entries = counts
        .iter()
        .map(|(label, count)| LabeledCount {
            label: label.clone(),
            count: *count,
        })
        .collect::<Vec<_>>();
    entries.sort_by(|left, right| {
        right
            .count
            .cmp(&left.count)
            .then_with(|| left.label.cmp(&right.label))
    });
    entries.truncate(limit);
    entries
}

fn sorted_count_labels(counts: &HashMap<String, usize>, limit: usize) -> Vec<String> {
    sorted_counts(counts, limit)
        .into_iter()
        .map(|entry| entry.label)
        .collect()
}

fn sorted_clusters(
    counts: &HashMap<String, usize>,
    examples: &HashMap<String, Vec<String>>,
    evidence_ids: &HashMap<String, Vec<String>>,
    limit: usize,
) -> Vec<SignalCluster> {
    let mut entries = counts
        .iter()
        .map(|(label, count)| SignalCluster {
            label: label.clone(),
            count: *count,
            examples: examples.get(label).cloned().unwrap_or_default(),
            evidence_ids: evidence_ids.get(label).cloned().unwrap_or_default(),
            confidence: cluster_confidence(*count).to_owned(),
        })
        .collect::<Vec<_>>();
    entries.sort_by(|left, right| {
        right
            .count
            .cmp(&left.count)
            .then_with(|| left.label.cmp(&right.label))
    });
    entries.truncate(limit);
    entries
}

fn cluster_confidence(count: usize) -> &'static str {
    if count >= 4 {
        "high"
    } else if count >= 2 {
        "medium"
    } else {
        "low"
    }
}

fn add_keyword_score(
    scores: &mut HashMap<SessionMode, usize>,
    text: &str,
    mode: SessionMode,
    keywords: &[&str],
    points: usize,
) {
    let matches = keywords
        .iter()
        .filter(|needle| text.contains(**needle))
        .count();
    if matches > 0 {
        *scores.entry(mode).or_default() += matches * points;
    }
}

fn extract_command_name(detail: &str) -> Option<String> {
    let command = detail.split('`').nth(1)?;
    canonical_tool_name(command.split_whitespace().next().unwrap_or_default()).into()
}

fn canonical_tool_name(name: &str) -> String {
    match name.trim() {
        "rg" | "ripgrep" => "rg".to_owned(),
        "cargo" | "cargo-test" => "cargo".to_owned(),
        "npm" | "pnpm" | "yarn" => name.trim().to_owned(),
        "git" => "git".to_owned(),
        "apply_patch" => "apply_patch".to_owned(),
        "sed" => "sed".to_owned(),
        "sqlite3" => "sqlite3".to_owned(),
        "python" | "python3" => "python".to_owned(),
        "rustc" => "rustc".to_owned(),
        other => other.to_owned(),
    }
}

fn build_work_areas(aggregated: &AggregatedInsightsData) -> Vec<WorkArea> {
    aggregated
        .active_projects
        .iter()
        .take(5)
        .map(|project| {
            let mode_summary = if project.dominant_modes.is_empty() {
                "mixed work".to_owned()
            } else {
                format!("mostly {}", project.dominant_modes.join(" + "))
            };
            let theme_summary = if project.top_themes.is_empty() {
                "general session flow".to_owned()
            } else {
                project.top_themes.join(", ")
            };
            let example_summary = if project.examples.is_empty() {
                format!("{} recent sessions tracked here.", project.session_count)
            } else {
                format!("Representative work: {}.", project.examples.join(" / "))
            };

            WorkArea {
                name: project.name.clone(),
                session_count: project.session_count,
                description: format!(
                    "{} recent sessions, {} around {}. {}",
                    project.session_count, mode_summary, theme_summary, example_summary
                ),
                modes: project.dominant_modes.clone(),
                themes: project.top_themes.clone(),
            }
        })
        .collect()
}

fn build_interaction_style(
    aggregated: &AggregatedInsightsData,
    work_areas: &[WorkArea],
) -> InteractionStyle {
    let total = aggregated.analyzed_session_count.max(1);
    let dominant_mode = aggregated
        .dominant_modes
        .first()
        .map(|mode| mode.label.as_str())
        .unwrap_or("mixed execution");
    let top_tools = aggregated
        .tool_usage
        .iter()
        .take(3)
        .map(|tool| tool.label.clone())
        .collect::<Vec<_>>();
    let dominant_context = aggregated
        .context_styles
        .first()
        .map(|style| style.label.as_str())
        .unwrap_or("Lean context");
    let dominant_area = work_areas
        .first()
        .map(|area| area.name.as_str())
        .unwrap_or("recent work");
    let dominant_share = work_areas
        .first()
        .map(|area| area.session_count as f64 / total as f64)
        .unwrap_or(0.0);

    let key_pattern = if aggregated.iterative_sessions * 2 >= total {
        "You usually stay in the thread and refine the work until it is usable.".to_owned()
    } else if dominant_share >= 0.6 {
        format!(
            "You concentrate on {dominant_area} until the session resolves into a tangible result."
        )
    } else {
        format!(
            "You use Codex across multiple projects, but the dominant rhythm is still {dominant_mode}."
        )
    };

    let mut sentences = vec![key_pattern.clone()];
    if !top_tools.is_empty() {
        sentences.push(format!(
            "Recent sessions are grounded in real execution: {} show up repeatedly as validation and delivery tools.",
            top_tools.join(", ")
        ));
    }
    if aggregated.sessions_with_context_overload * 3 >= total {
        sentences.push(
            "A visible share of sessions starts from pasted handoffs or dense references, so compact summaries and evidence grouping matter more than raw transcript length."
                .to_owned(),
        );
    } else {
        sentences.push(format!(
            "The dominant context style is {dominant_context}, which means session summaries can stay specific without carrying the full transcript forward."
        ));
    }
    if work_areas.len() > 1 {
        sentences.push(
            "Because recent work spans several projects, the useful memory layer is not just per-thread recap but cross-session patterns by project, mode, and theme."
                .to_owned(),
        );
    }

    InteractionStyle {
        narrative: sentences.join(" "),
        key_pattern,
    }
}

fn build_what_works(
    aggregated: &AggregatedInsightsData,
    work_areas: &[WorkArea],
) -> Vec<InsightCard> {
    let total = aggregated.analyzed_session_count.max(1);
    let mut cards = aggregated
        .recurring_success_patterns
        .iter()
        .take(3)
        .map(|cluster| InsightCard {
            title: cluster.label.clone(),
            detail: format!(
                "{} of {} analyzed sessions showed this pattern. {}",
                cluster.count,
                total,
                cluster
                    .examples
                    .first()
                    .map(|example| format!("Representative evidence: {example}"))
                    .unwrap_or_else(|| "This pattern repeats across unrelated sessions.".to_owned())
            ),
        })
        .collect::<Vec<_>>();

    if cards.is_empty() {
        cards.push(InsightCard {
            title: "Sessions produce reusable signals".to_owned(),
            detail: "Even when the transcript is noisy, the archive still captures repeatable requests, execution evidence, and delivery patterns worth preserving.".to_owned(),
        });
    }

    if let Some(area) = work_areas.first() {
        cards.push(InsightCard {
            title: "Project concentration strengthens memory".to_owned(),
            detail: format!(
                "{} is the busiest recent work area, which makes it easier to spot stable workflows instead of one-off anecdotes.",
                area.name
            ),
        });
    }

    cards.truncate(4);
    cards
}

fn build_friction_cards(aggregated: &AggregatedInsightsData) -> Vec<FrictionCard> {
    aggregated
        .recurring_frictions
        .iter()
        .take(4)
        .map(|cluster| FrictionCard {
            category: cluster.label.clone(),
            detail: format!(
                "{} analyzed sessions showed this pattern, so it is a recurring workflow issue rather than a one-off transcript quirk.",
                cluster.count
            ),
            examples: cluster.examples.clone(),
        })
        .collect()
}

fn build_suggestion_cards(
    aggregated: &AggregatedInsightsData,
    work_areas: &[WorkArea],
    friction: &[FrictionCard],
) -> Vec<SuggestionCard> {
    let mut suggestions = Vec::new();

    if friction
        .iter()
        .any(|item| item.category == "Execution friction")
    {
        suggestions.push(SuggestionCard {
            title: "Turn failure clusters into preflight checks".to_owned(),
            detail: "When the same commands and validations fail across sessions, capture setup checks early so implementation threads start from a known-good environment.".to_owned(),
        });
    }
    if friction
        .iter()
        .any(|item| item.category == "Context overload")
    {
        suggestions.push(SuggestionCard {
            title: "Collapse handoffs into smaller evidence packs".to_owned(),
            detail: "Recent context-heavy sessions suggest that summaries, snippets, and narrowed file references will travel better than full pasted documents.".to_owned(),
        });
    }
    if friction
        .iter()
        .any(|item| item.category == "Soft thread endings")
    {
        suggestions.push(SuggestionCard {
            title: "Ask for explicit finish states".to_owned(),
            detail: "A short final checkpoint like shipped / blocked / next step makes the archive far more truthful than inferring completion from partial commentary.".to_owned(),
        });
    }
    if work_areas.len() > 1 {
        let area = work_areas
            .first()
            .map(|item| item.name.as_str())
            .unwrap_or("project");
        suggestions.push(SuggestionCard {
            title: "Use project-scoped reports more aggressively".to_owned(),
            detail: format!(
                "Your recent archive spans multiple work areas. `codex-threads insights --project {area}` will give you a cleaner per-project memory layer when global summaries feel diluted."
            ),
        });
    }
    if aggregated.validated_sessions > 0 {
        suggestions.push(SuggestionCard {
            title: "Promote repeated good sessions into reusable workflows".to_owned(),
            detail: "When the same pattern keeps producing strong outcomes, capture it as a checklist or command recipe instead of rediscovering it thread by thread.".to_owned(),
        });
    }

    suggestions.truncate(4);
    suggestions
}

fn build_horizon_items(
    aggregated: &AggregatedInsightsData,
    work_areas: &[WorkArea],
) -> Vec<String> {
    let mut items = Vec::new();

    if aggregated.validated_sessions > 0 {
        items.push(
            "The next step is moving from narrative recap to repeatable workflows: a report should tell you which execution loops are mature enough to automate."
                .to_owned(),
        );
    }
    if aggregated.sessions_with_failures > 0 {
        items.push(
            "Recurring failure clusters can evolve into diagnostics, preflight checks, or project-specific guardrails instead of remaining buried in old transcripts."
                .to_owned(),
        );
    }
    if let Some(area) = work_areas.first() {
        items.push(format!(
            "{} already has enough repeated sessions to support a project memory layer that tracks modes, themes, and failure patterns over time.",
            area.name
        ));
    }
    if items.is_empty() {
        items.push(
            "As the archive grows, the biggest upgrade is separating per-session facts from cross-session truths so insights become queryable rather than anecdotal."
                .to_owned(),
        );
    }

    items.truncate(3);
    items
}

fn build_global_evidence(analyses: &[SessionAnalysis]) -> Vec<EvidenceItem> {
    analyses
        .iter()
        .take(6)
        .enumerate()
        .map(|(index, analysis)| {
            let fact = &analysis.fact;
            let facets = &analysis.facets;
            let title = fact
                .objective
                .clone()
                .or_else(|| fact.thread.summary.clone())
                .unwrap_or_else(|| format!("Session {}", &fact.thread.thread_id[..8]));
            let mut parts = Vec::new();
            if let Some(request) = fact.key_requests.first() {
                parts.push(format!("Request: {}", one_line(request, 160)));
            }
            if let Some(outcome) = &fact.outcome {
                parts.push(format!("Outcome: {}", one_line(outcome, 160)));
            }
            if let Some(action) = fact.key_actions.first() {
                parts.push(format!("Evidence: {}", one_line(&action.detail, 160)));
            }

            EvidenceItem {
                id: format!("evidence-{}", index + 1),
                thread_id: fact.thread.thread_id.clone(),
                title: one_line(&title, 96),
                detail: parts.join(" "),
                project: facets.project.clone(),
                mode: facets.primary_mode.label().to_owned(),
                themes: facets.themes.clone(),
                confidence: evidence_confidence(fact, facets).to_owned(),
            }
        })
        .collect()
}

fn evidence_confidence(fact: &SessionFact, facets: &SessionFacets) -> &'static str {
    if fact.command_successes > 0 && matches!(facets.outcome_strength, OutcomeStrength::Strong) {
        "high"
    } else if fact.outcome.is_some() || !fact.key_actions.is_empty() {
        "medium"
    } else {
        "low"
    }
}

fn build_at_a_glance(
    aggregated: &AggregatedInsightsData,
    work_areas: &[WorkArea],
    interaction_style: &InteractionStyle,
    friction: &[FrictionCard],
    suggestions: &[SuggestionCard],
) -> AtAGlance {
    let total = aggregated.analyzed_session_count.max(1);
    let dominant_area = work_areas
        .first()
        .map(|area| area.name.as_str())
        .unwrap_or("recent work");
    let strong_outcomes = aggregated
        .outcome_strengths
        .iter()
        .find(|entry| entry.label == "Strong outcome")
        .map(|entry| entry.count)
        .unwrap_or(0);
    let dominant_mode = aggregated
        .dominant_modes
        .first()
        .map(|entry| entry.label.as_str())
        .unwrap_or("mixed work");

    AtAGlance {
        whats_working: format!(
            "{} {} of {} analyzed sessions validated work with real commands, and {} landed as strong outcomes.",
            interaction_style.key_pattern, aggregated.validated_sessions, total, strong_outcomes
        ),
        whats_hindering: friction
            .first()
            .map(|card| format!("{} This repeats enough to distort cross-session reporting unless it is modeled explicitly.", card.detail))
            .unwrap_or_else(|| "The main limitation is still truthfulness: some sessions do real work but end without a crisp final state, so retrospective analysis has to infer too much.".to_owned()),
        quick_wins: suggestions
            .first()
            .map(|item| item.detail.clone())
            .unwrap_or_else(|| "Use project-scoped reports and tighter evidence snippets when you want a more precise memory layer.".to_owned()),
        ambitious_workflows: format!(
            "The strongest next workflow is a facet-based memory layer around {dominant_area}: once sessions are tagged by {dominant_mode}, themes, and friction patterns, the archive can surface trends instead of only summaries."
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
    evidence: &[EvidenceItem],
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
    writeln!(
        &mut text,
        "- **What's working:** {}",
        at_a_glance.whats_working
    )
    .expect("write to string");
    writeln!(
        &mut text,
        "- **What's hindering:** {}",
        at_a_glance.whats_hindering
    )
    .expect("write to string");
    writeln!(&mut text, "- **Quick wins:** {}", at_a_glance.quick_wins).expect("write to string");
    writeln!(
        &mut text,
        "- **Ambitious workflows:** {}",
        at_a_glance.ambitious_workflows
    )
    .expect("write to string");

    if !work_areas.is_empty() {
        writeln!(&mut text, "\n## What You Work On").expect("write to string");
        for area in work_areas {
            let chips = [
                format!("modes: {}", area.modes.join(", ")),
                format!("themes: {}", area.themes.join(", ")),
            ]
            .into_iter()
            .filter(|item| !item.ends_with(": "))
            .collect::<Vec<_>>()
            .join(" · ");
            writeln!(
                &mut text,
                "- **{}**: {}{}",
                area.name,
                area.description,
                if chips.is_empty() {
                    String::new()
                } else {
                    format!(" [{}]", chips)
                }
            )
            .expect("write to string");
        }
    }

    writeln!(&mut text, "\n## How You Use Codex").expect("write to string");
    writeln!(&mut text, "{}", interaction_style.narrative).expect("write to string");

    if !what_works.is_empty() {
        writeln!(&mut text, "\n## What Works").expect("write to string");
        for item in what_works {
            writeln!(&mut text, "- **{}**: {}", item.title, item.detail).expect("write to string");
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
            writeln!(&mut text, "- **{}**: {}", item.title, item.detail).expect("write to string");
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
            writeln!(
                &mut text,
                "- **{}** ({} · {}{}): {}",
                item.title,
                item.project,
                item.mode,
                if item.themes.is_empty() {
                    String::new()
                } else {
                    format!(" · {}", item.themes.join(", "))
                },
                item.detail
            )
            .expect("write to string");
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
    let nav = [
        ("#section-glance", "At a Glance"),
        ("#section-work", "What You Work On"),
        ("#section-usage", "How You Use Codex"),
        ("#section-wins", "What Works"),
        ("#section-friction", "Where Things Go Wrong"),
        ("#section-suggestions", "Suggestions"),
        ("#section-horizon", "On the Horizon"),
        ("#section-evidence", "Evidence"),
    ]
    .iter()
    .map(|(href, label)| format!("<a href=\"{}\">{}</a>", href, escape_html(label)))
    .collect::<Vec<_>>()
    .join("");

    let metric = |label: &str, value: String| {
        format!(
            "<div class=\"metric\"><div class=\"metric-label\">{}</div><div class=\"metric-value\">{}</div></div>",
            escape_html(label),
            escape_html(&value)
        )
    };

    let dominant_mode = report
        .aggregated
        .dominant_modes
        .first()
        .map(|item| item.label.clone())
        .unwrap_or_else(|| "Mixed".to_owned());
    let active_projects = report.aggregated.active_projects.len();
    let metrics = [
        metric(
            "Sessions analyzed",
            report.metadata.sessions_analyzed.to_string(),
        ),
        metric("Active projects", active_projects.to_string()),
        metric("Dominant mode", dominant_mode),
        metric(
            "Validated sessions",
            report.aggregated.validated_sessions.to_string(),
        ),
    ]
    .join("");

    let glance_cards = [
        ("What's working", &report.at_a_glance.whats_working, "section-wins"),
        (
            "What's hindering",
            &report.at_a_glance.whats_hindering,
            "section-friction",
        ),
        ("Quick wins", &report.at_a_glance.quick_wins, "section-suggestions"),
        (
            "Ambitious workflows",
            &report.at_a_glance.ambitious_workflows,
            "section-horizon",
        ),
    ]
    .iter()
    .map(|(label, detail, target)| {
        format!(
            "<article class=\"glance-card\"><div class=\"eyebrow\">{}</div><p>{}</p><a class=\"see-more\" href=\"#{}\">See section →</a></article>",
            escape_html(label),
            escape_html(detail),
            escape_html(target)
        )
    })
    .collect::<Vec<_>>()
    .join("");

    let recurring_themes = report
        .aggregated
        .recurring_themes
        .iter()
        .take(6)
        .map(|item| {
            format!(
                "<span class=\"chip\">{} · {}</span>",
                escape_html(&item.label),
                item.count
            )
        })
        .collect::<Vec<_>>()
        .join("");

    let work_areas = report
        .work_areas
        .iter()
        .map(|area| {
            let chips = area
                .modes
                .iter()
                .chain(area.themes.iter())
                .map(|item| format!("<span class=\"chip\">{}</span>", escape_html(item)))
                .collect::<Vec<_>>()
                .join("");
            format!(
                "<article class=\"card\"><div class=\"card-header\"><h3>{}</h3><span class=\"session-chip\">~{} sessions</span></div><p>{}</p><div class=\"chips\">{}</div></article>",
                escape_html(&area.name),
                area.session_count,
                escape_html(&area.description),
                chips
            )
        })
        .collect::<Vec<_>>()
        .join("");

    let what_works = report
        .what_works
        .iter()
        .map(|item| {
            format!(
                "<article class=\"card\"><h3>{}</h3><p>{}</p></article>",
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
                    "<ul class=\"examples\">{}</ul>",
                    item.examples
                        .iter()
                        .map(|example| format!("<li>{}</li>", escape_html(example)))
                        .collect::<Vec<_>>()
                        .join("")
                )
            };
            format!(
                "<article class=\"card warning\"><h3>{}</h3><p>{}</p>{}</article>",
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
                "<article class=\"card\"><h3>{}</h3><p>{}</p></article>",
                escape_html(&item.title),
                escape_html(&item.detail)
            )
        })
        .collect::<Vec<_>>()
        .join("");

    let horizon = report
        .on_the_horizon
        .iter()
        .map(|item| {
            format!(
                "<article class=\"card\"><p>{}</p></article>",
                escape_html(item)
            )
        })
        .collect::<Vec<_>>()
        .join("");

    let evidence = report
        .evidence
        .iter()
        .map(|item| {
            let chips = std::iter::once(item.project.clone())
                .chain(std::iter::once(item.mode.clone()))
                .chain(item.themes.clone().into_iter())
                .map(|label| format!("<span class=\"chip\">{}</span>", escape_html(&label)))
                .collect::<Vec<_>>()
                .join("");
            format!(
                "<article class=\"card evidence-card\"><h3>{}</h3><div class=\"chips\">{}</div><p>{}</p></article>",
                escape_html(&item.title),
                chips,
                escape_html(&item.detail)
            )
        })
        .collect::<Vec<_>>()
        .join("");

    format!(
        "<!doctype html><html><head><meta charset=\"utf-8\"><title>Codex Insights</title><style>:root{{--bg:#f5f1e8;--surface:#fffaf2;--surface-strong:#fff;--ink:#1f1c17;--muted:#6c655d;--line:#d9cfbf;--accent:#0f766e;--accent-soft:#d7eeeb;--warn:#9a3412;--warn-soft:#fce9df;--shadow:0 18px 50px rgba(31,28,23,.08)}}*{{box-sizing:border-box}}body{{margin:0;background:radial-gradient(circle at top,#fff8ef 0,#f5f1e8 45%,#efe5d6 100%);color:var(--ink);font-family:ui-sans-serif,system-ui,-apple-system,sans-serif;line-height:1.6}}main{{max-width:1120px;margin:0 auto;padding:40px 24px 72px}}header{{margin-bottom:28px}}h1,h2,h3{{margin:0 0 10px;color:#171411}}h1{{font-size:clamp(2.3rem,4vw,4rem);line-height:1}}h2{{font-size:1.5rem}}p{{margin:0 0 10px}}a{{color:var(--accent);text-decoration:none}}nav{{display:flex;flex-wrap:wrap;gap:10px;margin:22px 0}}nav a{{padding:8px 12px;border-radius:999px;background:rgba(255,255,255,.65);border:1px solid var(--line);font-size:.92rem}}section{{margin-top:34px}}.subtitle,.meta{{color:var(--muted)}}.hero{{display:grid;grid-template-columns:2fr 1fr;gap:18px;align-items:start}}.panel,.card,.glance-card,.metric{{background:rgba(255,250,242,.92);border:1px solid var(--line);border-radius:22px;box-shadow:var(--shadow)}}.panel{{padding:22px}}.metrics{{display:grid;grid-template-columns:repeat(4,minmax(0,1fr));gap:12px}}.metric{{padding:16px}}.metric-label{{font-size:.8rem;color:var(--muted);text-transform:uppercase;letter-spacing:.08em}}.metric-value{{font-size:1.6rem;font-weight:700;margin-top:6px}}.glance-grid,.card-grid{{display:grid;grid-template-columns:repeat(2,minmax(0,1fr));gap:14px}}.glance-card,.card{{padding:18px}}.warning{{background:var(--warn-soft);border-color:#efc6b5}}.eyebrow{{font-size:.78rem;letter-spacing:.08em;text-transform:uppercase;color:var(--muted);margin-bottom:8px}}.see-more{{display:inline-block;margin-top:8px;font-weight:600}}.chips{{display:flex;flex-wrap:wrap;gap:8px;margin-top:12px}}.chip,.session-chip{{display:inline-flex;align-items:center;padding:5px 10px;border-radius:999px;background:var(--accent-soft);color:#0b4f49;font-size:.85rem}}.session-chip{{background:#efe7da;color:#564b3f}}.card-header{{display:flex;justify-content:space-between;gap:12px;align-items:flex-start}}.narrative{{font-size:1.02rem}}.key-pattern{{margin-top:14px;padding:12px 14px;border-left:4px solid var(--accent);background:#eef7f5;border-radius:14px}}.examples{{margin:10px 0 0;padding-left:18px}}.examples li{{margin:6px 0}}.theme-strip{{margin-top:14px}}.evidence-card p{{margin-top:12px}}.section-stack{{display:grid;gap:14px}}@media (max-width:900px){{.hero,.glance-grid,.card-grid,.metrics{{grid-template-columns:1fr}}main{{padding:28px 16px 56px}}}}</style></head><body><main><header><div class=\"eyebrow\">Session-facet insights report</div><h1>Codex Insights</h1><p class=\"subtitle\">{} sessions scanned · {} analyzed</p><p class=\"meta\">HTML report: {}</p>{}</header><div class=\"hero\"><section id=\"section-glance\" class=\"panel\"><h2>At a Glance</h2><div class=\"glance-grid\">{}</div></section><aside class=\"panel\"><h2>Metrics</h2><div class=\"metrics\">{}</div><div class=\"theme-strip\"><div class=\"eyebrow\">Recurring themes</div><div class=\"chips\">{}</div></div></aside></div><section id=\"section-work\"><h2>What You Work On</h2><div class=\"card-grid\">{}</div></section><section id=\"section-usage\"><h2>How You Use Codex</h2><div class=\"panel narrative\"><p>{}</p><div class=\"key-pattern\"><strong>Key pattern:</strong> {}</div></div></section><section id=\"section-wins\"><h2>What Works</h2><div class=\"card-grid\">{}</div></section><section id=\"section-friction\"><h2>Where Things Go Wrong</h2><div class=\"card-grid\">{}</div></section><section id=\"section-suggestions\"><h2>Suggestions</h2><div class=\"card-grid\">{}</div></section><section id=\"section-horizon\"><h2>On the Horizon</h2><div class=\"section-stack\">{}</div></section><section id=\"section-evidence\"><h2>Evidence</h2><div class=\"card-grid\">{}</div></section></main></body></html>",
        report.metadata.sessions_scanned,
        report.metadata.sessions_analyzed,
        escape_html(&report.metadata.generated_report_path),
        nav,
        glance_cards,
        metrics,
        recurring_themes,
        work_areas,
        escape_html(&report.interaction_style.narrative),
        escape_html(&report.interaction_style.key_pattern),
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
