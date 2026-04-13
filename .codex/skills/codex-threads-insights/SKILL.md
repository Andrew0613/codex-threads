---
name: codex-threads-insights
description: Generate Codex usage insight reports from the codex-threads CLI, using `--json insights` and `data.briefing` as the primary source of truth.
---

# codex-threads-insights

Use this when you need a report about how Codex was used across recent sessions.

This skill is for the report-writing workflow, not thread lookup. If you need to find or read one session first, use `codex-threads`.

## Truth Source

Use `--json insights` and treat `data.briefing` as the canonical backend payload.

- Prefer:
  - `data.briefing.summary`
  - `data.briefing.patterns`
  - `data.briefing.evidence`
  - `data.briefing.example_sessions`
  - `data.briefing.uncertainties`
  - `data.briefing.modeling_notes`
- Treat `data.content` as a heuristic draft only.
- Do not use `example_sessions.len()` as the analyzed-session denominator; use `sessions_analyzed`.

## Default Flow

1. Verify the CLI works.

```bash
cargo run -- --json doctor
```

2. Refresh the local index when needed.

```bash
cargo run -- sync
```

3. Generate the structured insights payload.

```bash
cargo run -- --json insights --limit 20
```

4. For one project, use:

```bash
cargo run -- --json insights --project <project-name> --limit 20
```

## How To Write The Report

- Base conclusions on recurring patterns, not one sample session.
- Use project/mode/theme/friction clusters as stronger signals than raw tool frequency.
- Use `confidence`, `classification_notes`, and `uncertainties` to calibrate claims.
- Use `pattern.evidence_ids` to connect narrative claims back to `evidence`.
- Keep quoted evidence short; summarize long transcript fragments instead of repeating them.
- If `data.content` conflicts with `data.briefing`, trust `data.briefing`.

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
