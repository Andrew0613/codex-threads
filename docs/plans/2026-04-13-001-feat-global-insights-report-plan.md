---
title: feat: Add global insights report
type: feat
status: active
date: 2026-04-13
---

# feat: Add global insights report

## Overview

Add a top-level `insights` command to `codex-threads` that analyzes many Codex sessions at once and produces a shareable report, rather than only per-thread recap output. The command should reuse the existing local SQLite index over `~/.codex/sessions`, derive deterministic multi-thread analytics, and render both a terminal-friendly summary and an HTML report path. The goal is to reproduce the practical shape of Claude Code's `/insights`: a usage report built from session history that surfaces work areas, interaction patterns, friction, and recommendations.

## Problem Frame

The current CLI is effective at single-thread retrieval (`threads read`, `events summary`, `threads summarize`, `threads insight`) but it cannot answer global questions such as "what patterns exist across my recent Codex usage?" or "what does my history suggest I should improve?" Claude Code's `/insights` command solves exactly this by scanning many sessions, aggregating behavior, and generating a multi-section report. `codex-threads` needs the same global analysis layer so it can move from thread browser to actual memory analytics tool.

## Requirements Trace

- R1. Add a top-level global insights command that analyzes multiple sessions, not just one thread.
- R2. Reuse the existing local index and session parsing model instead of requiring external services or a new data store.
- R3. Produce a stable report structure with sections comparable to Claude Code `/insights`, including at least: at-a-glance summary, project/work areas, interaction style, what works, friction, and recommendations.
- R4. Support practical filtering for real use, at minimum recent-session limits and optional project scoping.
- R5. Provide both terminal output and machine-readable JSON; include an HTML report artifact to make the result feel like a real report rather than raw stats.
- R6. Keep the implementation deterministic and local-first so it works on existing Codex archives without API dependencies.
- R7. Cover the new behavior with tests and verify it against real session data from `~/.codex/sessions`.

## Scope Boundaries

- No remote collection or homespace support in this iteration.
- No LLM-based facet extraction or external inference APIs in this iteration.
- No attempt to reach byte-for-byte parity with Claude Code HTML or exact wording.
- Do not remove or replace existing single-thread commands; global insights should be additive.

## Context & Research

### Relevant Code and Patterns

- `src/main.rs`
  Current command tree, SQLite schema, rendering helpers, session parsing, per-thread analytics, and tests all live in one file today.
- `src/main.rs`
  Existing reusable patterns:
  - `read_thread(...)` for clean thread loading
  - `summarize_events(...)` / `summarize_event_stream(...)` for noise suppression
  - `build_thread_digest(...)` and `build_thread_insight(...)` for transforming raw session material into higher-level summaries
  - `render_*` helpers for consistent terminal and JSON contracts
- `README.md`
  Existing command documentation pattern and JSON contract description.

### Institutional Learnings

- No `docs/solutions/` corpus exists in this repository today, so there are no local institutional learnings to carry forward.

### External References

- `../claude-code-source-code/src/commands.ts`
  Confirms the feature shape is a lazy-loaded top-level `insights` command, not a thread subcommand.
- `../claude-code-source-code/src/commands/insights.ts`
  Key reference behavior:
  - scans many sessions
  - filters meta/minimal sessions
  - aggregates usage metrics and session summaries
  - generates multiple report sections in parallel
  - renders terminal summary plus HTML report
- `../claude-code-source-code/src/commands/insights.ts`
  Important section targets to mirror in spirit:
  - `at_a_glance`
  - `project_areas`
  - `interaction_style`
  - `what_works`
  - `friction_analysis`
  - `suggestions`
  - `on_the_horizon`

## Key Technical Decisions

- Add a top-level `insights` command rather than extending `threads insight`.
  Rationale: Claude's feature is global usage analysis, not single-thread recap. Keeping it top-level makes the mental model clearer and preserves the existing thread-level tools.

- Build deterministic analytics from indexed thread/message/event data instead of using an LLM.
  Rationale: `codex-threads` should remain local-first and reproducible. The current SQLite index already contains enough signal to generate a credible first report.

- Introduce a multi-session analytics layer with explicit typed report structures.
  Rationale: the current code can summarize one thread, but global insights need a new intermediate model for aggregated metrics, sections, and rendered artifacts.

- Generate an HTML report artifact in addition to terminal output.
  Rationale: Claude's `/insights` feels substantial partly because it produces a shareable report. HTML is the most direct way to reproduce that feeling without adding frontend dependencies.

- Filter out obviously low-signal sessions before aggregation.
  Rationale: minimal or noisy sessions distort the report. The implementation should exclude trivial threads based on simple observable signals such as message counts and absent substantive user turns.

- Keep report copy heuristic-driven and evidence-backed.
  Rationale: without an LLM, the report must stay grounded in observable patterns and cite concrete evidence from projects, commands, outcomes, and repeated user behaviors.

## Open Questions

### Resolved During Planning

- Should this be a thread subcommand or a top-level command?
  Resolution: top-level `insights`.

- Should the feature depend on external APIs or an LLM?
  Resolution: no; use deterministic local analytics for this iteration.

- Should the feature produce only text, or a report artifact too?
  Resolution: produce terminal output, JSON, and a generated HTML report path.

### Deferred to Implementation

- Exact heuristics for identifying "minimal" vs substantive sessions.
  Why deferred: this is best tuned against real indexed data during implementation.

- Exact section wording and ranking heuristics for suggestions.
  Why deferred: final quality depends on seeing real aggregate outputs and adjusting the rules pragmatically.

- Exact HTML layout and styling.
  Why deferred: the report structure is known, but the final presentation should be refined once the content shape exists.

## High-Level Technical Design

> *This illustrates the intended approach and is directional guidance for review, not implementation specification. The implementing agent should treat it as context, not code to reproduce.*

```text
sessions index (threads/messages/events tables)
  -> candidate session selection (recent/project filters, minimal-session pruning)
  -> per-session derived stats
       - message counts, user follow-ups, outcome signal, failures, commands, projects
  -> aggregate report model
       - totals
       - work areas
       - interaction patterns
       - success patterns
       - friction patterns
       - suggestions / next moves
  -> report renderers
       - terminal markdown/text
       - JSON payload
       - HTML artifact on disk
```

## Implementation Units

- [ ] **Unit 1: Define the global insights command and report contract**

**Goal:** Add the new top-level CLI entrypoint, arguments, report data structures, and render contract.

**Requirements:** R1, R4, R5

**Dependencies:** None

**Files:**
- Modify: `src/main.rs`
- Modify: `README.md`
- Test: `src/main.rs`

**Approach:**
- Add a new top-level `Insights` command with arguments such as recent-session `--limit`, optional `--project`, and optional output path override for HTML.
- Define typed structs for aggregate analytics and final report sections, separate from single-thread `ThreadInsight`.
- Keep JSON output stable and explicit, following the existing `{"ok": true, "command": ..., "data": ...}` contract.

**Patterns to follow:**
- Existing command wiring in `src/main.rs`
- `render_thread_insight(...)`
- `render_events_summary(...)`

**Test scenarios:**
- Happy path: invoking the new command with indexed data returns a JSON payload tagged as `insights`.
- Happy path: limiting sessions reduces the analyzed set deterministically.
- Edge case: project filter with no matching sessions returns a valid empty-report shape rather than crashing.
- Error path: missing thread/session data needed for a report yields a user-facing error only when no analyzable sessions remain.

**Verification:**
- The CLI exposes `codex-threads insights ...` and returns terminal + JSON output without breaking existing commands.

- [ ] **Unit 2: Build multi-session analytics and session filtering**

**Goal:** Turn indexed threads/messages/events into a reusable aggregate model for global insights.

**Requirements:** R1, R2, R4, R6

**Dependencies:** Unit 1

**Files:**
- Modify: `src/main.rs`
- Test: `src/main.rs`

**Approach:**
- Query the existing SQLite index for candidate sessions using recency and optional project filters.
- Add deterministic filtering for low-signal sessions using message counts, substantive user messages, and similar observable traits.
- Derive per-session stats such as:
  - user/assistant message counts
  - recent project/work area
  - notable command successes/failures
  - presence of multi-step refinement
  - likely completion/outcome signals from assistant final answers
- Aggregate those per-session stats into cross-session totals and ranked buckets.

**Technical design:** *(directional guidance)*
- Treat each thread as the unit of analysis.
- Build "session facts" first, then aggregate; do not mix raw SQL reads directly into report prose generation.

**Patterns to follow:**
- `recent_threads(...)`
- `search_messages(...)`
- `summarize_event_stream(...)`
- SQLite row-mapping style already used throughout `src/main.rs`

**Test scenarios:**
- Happy path: a dataset with multiple projects produces ranked project/work area counts.
- Happy path: sessions with assistant final answers contribute outcome/success signals.
- Edge case: trivial sessions are excluded when they contain only tiny or non-substantive user messages.
- Edge case: sessions with no final answer still contribute usage stats without corrupting outcome summaries.
- Error path: failed commands are counted as friction rather than causing report generation failure.
- Integration: report generation across threads uses existing indexed events/messages rather than reparsing raw JSONL files.

**Verification:**
- The code can produce a stable aggregate analytics model from the SQLite index for real local data.

- [ ] **Unit 3: Generate insight sections from deterministic heuristics**

**Goal:** Produce report sections that feel like `/insights`, grounded in observable patterns instead of raw logs.

**Requirements:** R3, R6

**Dependencies:** Unit 2

**Files:**
- Modify: `src/main.rs`
- Test: `src/main.rs`

**Approach:**
- Add section generators for:
  - at-a-glance summary
  - work/project areas
  - interaction style
  - what works
  - friction analysis
  - suggestions / next steps
  - optional forward-looking "on the horizon" section if enough signal exists
- Base each section on explicit aggregate signals and evidence lists.
- Prefer grounded language ("you often iterate by ...", "recent sessions show ...") over generic filler.

**Execution note:** Implement this unit characterization-first against real report output; adjust heuristics only when the generated content is clearly noisy or generic.

**Patterns to follow:**
- `build_thread_digest(...)`
- `build_thread_insight(...)`
- Claude reference structure in `../claude-code-source-code/src/commands/insights.ts`

**Test scenarios:**
- Happy path: mixed project sessions produce at least one work-area section and one suggestion.
- Happy path: repeated multi-step user follow-ups produce an interaction-style insight.
- Edge case: sparse data still yields a valid report with fewer sections instead of placeholder noise.
- Edge case: conflicting signals do not duplicate identical insights across sections.
- Error path: absent friction signals omit or minimize the friction section rather than inventing problems.

**Verification:**
- Running the command on real local data yields readable sections that describe usage patterns, not just counts.

- [ ] **Unit 4: Render HTML report and terminal summary**

**Goal:** Make the insights output feel like a full report rather than only a CLI dump.

**Requirements:** R5

**Dependencies:** Unit 3

**Files:**
- Modify: `src/main.rs`
- Modify: `README.md`
- Test: `src/main.rs`

**Approach:**
- Add a lightweight HTML renderer with clear sections and evidence blocks.
- Persist the generated report under a predictable local path inside the existing Codex home area, unless an explicit output path is supplied.
- Return the HTML path in JSON and mention it in the terminal output.

**Patterns to follow:**
- Existing `RenderedOutput` contract
- Claude reference: report generation and saved report path behavior in `../claude-code-source-code/src/commands/insights.ts`

**Test scenarios:**
- Happy path: the command writes an HTML report and returns its path.
- Edge case: repeated runs overwrite or refresh the report predictably.
- Edge case: report generation with partial sections still yields valid HTML.
- Error path: invalid output directory surfaces a filesystem error cleanly.

**Verification:**
- A real HTML report file is created locally and can be opened after command execution.

- [ ] **Unit 5: Documentation, regression coverage, and real-data validation**

**Goal:** Lock the feature in with tests, docs, and real session verification.

**Requirements:** R7

**Dependencies:** Units 1-4

**Files:**
- Modify: `README.md`
- Modify: `src/main.rs`
- Test: `src/main.rs`

**Approach:**
- Add focused unit tests for aggregation, filtering, section generation, and HTML rendering.
- Update README examples and notes to include the new top-level insights workflow.
- Validate against real `~/.codex/sessions` data to ensure output quality is materially better than the current single-thread `threads insight`.

**Patterns to follow:**
- Existing inline test style in `src/main.rs`
- Existing README command list structure

**Test scenarios:**
- Happy path: end-to-end fixture data generates a stable insights report shape.
- Edge case: project filters and session limits are reflected in the report metadata.
- Edge case: empty evidence buckets do not break report rendering.
- Integration: real local data run succeeds on the user's machine with the installed binary.

**Verification:**
- Tests pass, README reflects shipped behavior, and a real local run produces a report that is clearly global rather than thread-local.

## System-Wide Impact

- **Interaction graph:** new top-level command depends on the existing index, message cleaning, event summarization, and render pipeline.
- **Error propagation:** SQLite query or report file write failures should surface as command errors through the existing `run(...) -> RenderedOutput` path.
- **State lifecycle risks:** the HTML report path becomes a new local artifact; writes should be deterministic and overwrite-safe.
- **API surface parity:** global insights should complement, not replace, `threads summarize` and `threads insight`.
- **Integration coverage:** real-data validation must prove that aggregate analytics uses indexed threads/messages/events coherently.
- **Unchanged invariants:** the SQLite schema for existing commands remains compatible, and existing thread/message/event commands should keep their current output contracts.

## Risks & Dependencies

| Risk | Mitigation |
|------|------------|
| Heuristic copy becomes generic or noisy | Keep every section grounded in ranked evidence and real aggregate signals; validate against real local sessions |
| Minimal-session filtering removes useful sessions | Make the filter conservative and cover it with fixtures plus real-data checks |
| HTML report becomes too heavy for a single-file Rust CLI | Keep rendering self-contained and static; avoid templating dependencies |
| Existing single-thread insight logic bleeds into the global report and confuses users | Separate aggregate report structs and top-level command names clearly |

## Documentation / Operational Notes

- Update the README command list and usage examples for the new `insights` command.
- No production monitoring is required; this is a local developer CLI. Validation is via tests and real local session runs.

## Sources & References

- Related code: `src/main.rs`
- Reference implementation: `../claude-code-source-code/src/commands.ts`
- Reference implementation: `../claude-code-source-code/src/commands/insights.ts`
