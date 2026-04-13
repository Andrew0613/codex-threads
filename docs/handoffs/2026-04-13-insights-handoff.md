# Handoff: Complete `insights` Parity Work

## Context

Repo: [codex-threads](/Users/puyuandong613/workspace/opensource/codex-threads)  
Current commit: `f0b5d42 feat: add global codex insights report`

This repo already has a working top-level `insights` command:

```bash
codex-threads insights --limit 20
codex-threads insights --project opensource --limit 20
codex-threads --json insights --project opensource --limit 2
```

It also writes an HTML artifact by default:

- [report.html](/Users/puyuandong613/.codex/codex-threads/insights/report.html)

There is also a thread-level layer already implemented:

- `threads summarize`
- `threads insight`
- `events summary`
- `messages search` snippet mode
- `threads recent`

The implementation plan used for the current version is here:

- [2026-04-13-001-feat-global-insights-report-plan.md](/Users/puyuandong613/workspace/opensource/codex-threads/docs/plans/2026-04-13-001-feat-global-insights-report-plan.md)

## What Exists Today

Main implementation file:

- [src/main.rs](/Users/puyuandong613/workspace/opensource/codex-threads/src/main.rs)

Current `insights` pipeline:

1. `candidate_threads_for_insights(...)`
2. `build_session_fact(...)`
3. `select_insight_facts(...)`
4. `build_global_insights_report(...)`
5. `render_global_insights_content(...)`
6. `generate_insights_html(...)`

Important helper layers already present:

- `summarize_events(...)`
- `build_thread_digest(...)`
- `build_thread_insight(...)`
- `sanitize_tool_output_issue(...)`
- `clean_thread_message(...)`

Current tests already cover:

- parser/index roundtrip
- snippet search behavior
- thread digest/insight rendering
- event summary behavior
- basic global insights helpers

Current test status:

```bash
cargo test
```

passes with 14 tests.

## What The Current Version Gets Right

- There is now a real top-level multi-session report, not just single-thread recap.
- Unscoped reports no longer get totally dominated by one busy project; they over-fetch and round-robin by project.
- Tool-output wrapper noise like `Chunk ID`, `Wall time`, raw shell trace, and similar event clutter is cleaned before being treated as evidence.
- The command already produces useful high-level sections:
  - `At a Glance`
  - `What You Work On`
  - `How You Use Codex`
  - `What Works`
  - `Where Things Go Wrong`
  - `Suggestions`
  - `On the Horizon`
  - `Evidence`

## What Is Still Missing

The current implementation is still heuristic-heavy. It is a good report generator, but not yet true Claude Code `/insights` parity.

The 3 biggest remaining gaps are:

1. Semantic facet extraction is missing.
Current code mostly derives conclusions from counts and simple text heuristics.
Claude Code first extracts structured per-session facets, then aggregates those facets across many sessions.

2. Cross-session analysis is still shallow.
The current report can say what projects are active, whether commands succeed, and whether context is heavy.
It still cannot reliably answer:
  - what themes are increasing over time
  - what failure modes repeat across unrelated threads
  - which collaboration patterns correlate with good outcomes
  - which sessions are exploratory vs executional vs planning vs review

3. Presentation is still thin.
The HTML report exists, but it is still a text-first rendering.
Claude Code `/insights` has a more deliberate section model, better data shaping, and richer report presentation.

## Claude Code `/insights` Reference

Local source reference:

- [insights.ts](/Users/puyuandong613/workspace/opensource/claude-code-source-code/src/commands/insights.ts)

Relevant anchors from that file:

- `type SessionFacets`
- `type AggregatedData`
- `aggregateData(...)`
- `generateParallelInsights(...)`
- `project_areas`
- `friction_analysis`
- `at_a_glance`
- `generateHtmlReport(...)`

High-level Claude `/insights` shape:

1. Load many sessions.
2. Filter duplicate/meta/low-value sessions.
3. Extract structured `SessionFacets` for each session.
4. Aggregate those facets into `AggregatedData`.
5. Generate report sections in parallel.
6. Render a richer HTML report and a user-facing summary.

That is the main conceptual gap: this repo currently does steps 1, 2, 4, 5, 6 in simplified form, but step 3 is still weak.

## The Goal For The Next Agent

Do not keep polishing local heuristics in place forever.
The goal is to move `codex-threads insights` from:

`recent-thread heuristic report`

to:

`session-facet based multi-session analysis engine`

The next agent should aim for:

- stronger per-session classification
- stronger cross-session aggregation
- better section quality
- better HTML output
- better report truthfulness

## Recommended Implementation Direction

### 1. Add a real per-session facet layer

Introduce a new internal struct, separate from `SessionFact`, for example:

```rust
struct SessionFacets {
    session_id: String,
    project: String,
    primary_mode: SessionMode,
    themes: Vec<String>,
    tools_used: Vec<String>,
    success_signals: Vec<String>,
    friction_signals: Vec<String>,
    context_style: ContextStyle,
    outcome_strength: OutcomeStrength,
}
```

Minimum useful fields:

- `primary_mode`
  - research
  - planning
  - implementation
  - debugging
  - review
  - exploratory
- `themes`
- `tools_used`
- `success_signals`
- `friction_signals`
- `outcome_strength`

This should be derived from:

- cleaned user messages
- final assistant outcome
- summarized events
- thread metadata like `project_name`, `cwd`, `message_count`

Do this deterministically first.
Do not block on adding an LLM step.

### 2. Separate aggregation from rendering

Right now the section builders derive conclusions directly from `SessionFact`.
Refactor toward:

1. raw indexed thread
2. `SessionFacets`
3. `AggregatedInsightsData`
4. section renderers

This will make the report less repetitive and much easier to improve.

Suggested aggregated fields:

- active_projects
- dominant_modes
- recurring_themes
- recurring_frictions
- recurring_success_patterns
- command_failure_clusters
- context_overload_clusters
- example_sessions

### 3. Upgrade section logic

Current sections are usable but still generic.
The next version should make each section more specific:

- `What You Work On`
  - should summarize project + mode + theme, not just project + objective snippet
- `How You Use Codex`
  - should describe collaboration style from repeated session patterns
- `What Works`
  - should be supported by repeated patterns, not just counts
- `Where Things Go Wrong`
  - should cluster failure modes
- `Suggestions`
  - should come from repeated frictions
- `On the Horizon`
  - should infer likely next product/workflow opportunities from the archive

### 4. Improve HTML report quality

Current HTML is acceptable but basic.
Upgrade it to include:

- summary cards
- clearer section navigation
- grouped evidence
- project/mode/theme chips
- a small metrics panel
- clearer distinction between evidence and interpretation

Do not over-design it, but it should feel like a report, not terminal text pasted into HTML.

### 5. Keep the CLI contract stable

Preserve:

- `codex-threads insights --limit N`
- `codex-threads insights --project NAME`
- `--json`
- default HTML write behavior

It is fine to add:

- `--since 30d`
- `--format html|text`
- `--output`
- `--include-modes`

But do not break the existing commands.

## Suggested Work Order

1. Read current implementation in [src/main.rs](/Users/puyuandong613/workspace/opensource/codex-threads/src/main.rs).
2. Read Claude reference points in [insights.ts](/Users/puyuandong613/workspace/opensource/claude-code-source-code/src/commands/insights.ts).
3. Add `SessionFacets` and deterministic extraction.
4. Add `AggregatedInsightsData`.
5. Rewrite section builders to consume aggregated data.
6. Improve HTML rendering.
7. Add tests for new facet extraction and aggregation behavior.
8. Run on real local session data and tune until the output becomes materially more specific.

## Concrete Acceptance Criteria

The next version should satisfy all of these:

1. Running `codex-threads insights --limit 20` produces a report that is visibly more specific than the current version.
2. The report can distinguish at least 3 session modes reliably on real data.
3. `Where Things Go Wrong` clusters failures by type instead of only listing raw examples.
4. `What Works` cites repeated collaboration patterns, not only aggregate counts.
5. The HTML report is clearly richer than the current plain text layout.
6. Tests cover:
   - facet extraction
   - aggregation
   - mode classification
   - failure clustering
   - HTML/report structure

## Current Real-Data Behavior

A recent project-scoped run:

```bash
codex-threads insights --project opensource --limit 20
```

currently reports:

- strong in-thread iteration
- real-command validation
- one recurring execution-friction cluster around OpenCLI/Twitter failures
- one recurring context-overload signal from long pasted references

This is directionally correct.
The next step is to make the report less generic and more causally informative.

## Recommended First Task For The Next Agent

Start by extracting a real `SessionFacets` layer.

If that layer is good, everything else becomes much easier:

- aggregation gets cleaner
- sections get more specific
- tests become easier to write
- HTML becomes easier to structure

Without that layer, the implementation will keep accumulating ad hoc heuristics.
