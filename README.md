# codex-threads

`codex-threads` turns `~/.codex/sessions` into a small local index you can search from any repo.

It is designed for the workflow described in Nick Baumann's post: find an old thread, resolve the right session id, then read the cleaned conversation or recent event stream without handing the raw archive back to Codex every time.

## What It Does

- `codex-threads --json sync`
  Scans `~/.codex/sessions/**/*.jsonl` and indexes thread metadata, readable user/assistant messages, and event summaries into a local SQLite database.
- `codex-threads --json insights --limit 50`
  Analyzes recent sessions globally and generates a report with work areas, interaction patterns, friction, suggestions, and an HTML artifact.
- `codex-threads --json messages search "build a CLI" --limit 20`
  Searches cleaned message text, returning compact match snippets instead of full long messages.
- `codex-threads --json threads resolve "tweet idea" --limit 20`
  Finds likely matching threads by project/cwd/path matches first, then message hits.
- `codex-threads --json threads recent --limit 20 --cwd /Users/.../workspace/opensource`
  Lists the most recent threads, with optional project or cwd filtering.
- `codex-threads --json threads read <session-id>`
  Reads the cleaned conversation for one thread and collapses large pasted skill/context blocks by default.
- `codex-threads --json threads summarize <session-id>`
  Produces a deterministic thread recap with objective, key user requests, notable actions, and outcome.
- `codex-threads --json threads insight <session-id>`
  Produces an insight-style note from one thread: a concrete summary, inferred takeaways, evidence, and next steps.
- `codex-threads --json events read <session-id> --limit 50`
  Reads the recent event stream for one thread.
- `codex-threads --json events summary <session-id> --limit 50`
  Compresses the recent event stream into high-signal command/tool/error milestones.
- `codex-threads --json doctor`
  Verifies session root and index health.

## Install

```bash
make install-local
```

That installs the binary into `~/.local/bin`.

## Usage

```bash
codex-threads --json sync
codex-threads --json insights --limit 50
codex-threads --json insights --project opensource --limit 20
codex-threads --json threads recent --limit 10 --project opensource
codex-threads --json messages search "build a CLI" --limit 20
codex-threads --json threads resolve "tweet idea" --limit 20
codex-threads --json threads read 019d8510-9b67-7ff0-914c-cc313085e394
codex-threads --json threads summarize 019d8510-9b67-7ff0-914c-cc313085e394
codex-threads --json threads insight 019d8510-9b67-7ff0-914c-cc313085e394
codex-threads --json threads read 019d8510-9b67-7ff0-914c-cc313085e394 --raw
codex-threads --json events read 019d8510-9b67-7ff0-914c-cc313085e394 --limit 50
codex-threads --json events summary 019d8510-9b67-7ff0-914c-cc313085e394 --limit 50
```

## JSON Contract

Under `--json`, every command returns:

```json
{
  "ok": true,
  "command": "threads.read",
  "data": {}
}
```

On failure:

```json
{
  "ok": false,
  "error": {
    "message": "thread not found: ..."
  }
}
```

## Notes

- The index lives at `~/.codex/codex-threads/index.sqlite3` by default.
- `insights` writes an HTML report to `~/.codex/codex-threads/insights/report.html` by default unless `--output` is supplied.
- Unscoped `insights` over-fetches recent sessions and then round-robins across projects so one busy repo does not drown out the whole report.
- The tool indexes readable `response_item.message` records and keeps the rawer stream for `events read`.
- `threads read` is clean-by-default. Use `--raw` if you want the full stored message text.
- `threads summarize` is deterministic and heuristic-driven; it is meant to be a fast recap, not a model-written narrative.
- `threads insight` sits one level above `summarize`: it turns the recap into a reusable note with takeaways and follow-up suggestions.
- `insights` sits above the thread-level tools: it looks across many sessions and produces a Claude `/insights`-style usage report.
- `messages search` is snippet-first by default so long pasted prompts do not dominate the result list.
- `events summary` keeps command completions, meaningful tool edits, and failures while suppressing token counts, reasoning, and most raw tool output.
- This first version prefers a boring contract over perfect ranking. The important property is that it is deterministic, scriptable, and fast enough to reuse from future Codex threads.
