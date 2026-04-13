---
name: codex-threads-insights
description: Generate Codex usage insight reports from the codex-threads CLI, using `--json insights` with `data.source_contract`, `data.trace`, and `data.briefing` as the canonical truth layers.
---

# codex-threads-insights

Use this when you need a report about how Codex was used across recent sessions.

This skill is for the report-writing workflow, not thread lookup. If you need to find or read one session first, use `codex-threads`.

## Truth Source

Use `--json insights` and treat `data.source_contract`, `data.trace`, and `data.briefing` as the canonical backend payload.

- Prefer:
  - `data.source_contract`
  - `data.trace`
  - `data.briefing.summary`
  - `data.briefing.patterns`
  - `data.briefing.evidence`
  - `data.briefing.example_sessions`
  - `data.briefing.uncertainties`
  - `data.briefing.modeling_notes`
- `data.metadata` and `data.aggregated` are derived overview fields from the same run. They are safe for quick summaries and cross-checks, but they are not the primary evidence layer.
- Treat `data.heuristic_draft` as a heuristic draft only.
- Do not use `example_sessions.len()` as the analyzed-session denominator; use `sessions_analyzed`.
- Do not assume evidence coverage is exhaustive; confirm `data.trace.canonical_receipt.evidence_item_count`, `evidence_item_cap`, and the actual `evidence_ids` present in this payload.

## Default Flow

1. Verify the CLI works.

```bash
cargo run -- --json doctor
```

2. Refresh the local index when needed.

```bash
cargo run -- sync
```

Run `sync` when the doctor output says the index is missing or stale, when recent sessions you expect are absent from `insights`, or after archive files changed since the last report run. Do not run it by reflex if `doctor` is healthy and the payload already reflects the sessions you need.

3. Generate the structured insights payload.

```bash
cargo run -- --json insights --limit 20
```

4. For one project, use:

```bash
cargo run -- --json insights --project <project-name> --limit 20
```

## Canonical Consumption Order

1. Read `data.source_contract`.
2. Read `data.trace` and lock:
   - valid evidence ids
   - valid recurring friction labels
   - valid recurring success labels
   - current analyzed-session denominator
   - whether `example_sessions` is sample-only and what the current sample cap is
   - whether `evidence` is sample-only and what the current evidence cap is
3. Read `data.briefing` for the actual fact layer.
4. Use `data.metadata` and `data.aggregated` only for overview/cross-check purposes.
5. Only after the narrative is already formed, optionally read `data.heuristic_draft` for wording ideas.

## How To Write The Report

- Base conclusions on recurring patterns, not one sample session.
- Use project/mode/theme/friction clusters as stronger signals than raw tool frequency.
- Use `confidence`, `classification_notes`, and `uncertainties` to calibrate claims.
- Use `pattern.evidence_ids` to connect narrative claims back to `evidence`.
- Keep quoted evidence short; summarize long transcript fragments instead of repeating them.
- If `data.heuristic_draft` conflicts with `data.trace` or `data.briefing`, trust `data.trace` and `data.briefing`.
- If `data.metadata` or `data.aggregated` appears to disagree with `data.trace` or `data.briefing`, trust `data.trace` and `data.briefing`.

## Recommended Sections

- `At a Glance`
- `What You Work On`
- `How You Use Codex`
- `What Works`
- `Where Things Go Wrong`
- `Suggestions`
- `On the Horizon`
- `Evidence`

## Guardrails

- Do not parse the human-readable CLI output in automation.
- Do not treat tool counts as a full explanation of behavior.
- Do not hide uncertainty when `Low execution evidence` or `Context overload` is high.
- Do not overfit on one active project if the analyzed set spans several projects.
- Do not cite any evidence id that is not present in `data.trace.canonical_receipt.evidence_ids`.
- Treat evidence ids as thread-bound identifiers from the current payload, not positional ranks.
- Do not assume every analyzed session has canonical evidence coverage; confirm `evidence_item_count` and `evidence_item_cap` before generalizing from `briefing.evidence`.
- Do not cite any recurring pattern that is not present in `data.trace`.
- Do not assume `example_sessions` is exhaustive; confirm `data.trace.canonical_receipt.example_sessions_are_samples` and use `example_session_cap` only as display metadata.
- Do not treat `example_sessions` as the full analyzed set; they are display samples only.
- Do not treat `data.aggregated` as a substitute for evidence linkage; use it for summary/cross-check only.
