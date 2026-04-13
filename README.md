# codex-threads

`codex-threads` turns `~/.codex/sessions` into a small local index you can search from any repo.

It is designed for the workflow described in Nick Baumann's post: find an old thread, resolve the right session id, then read the cleaned conversation or recent event stream without handing the raw archive back to Codex every time.

## What It Does

- `codex-threads --json sync`
  Scans `~/.codex/sessions/**/*.jsonl` and indexes thread metadata, readable user/assistant messages, and event summaries into a local SQLite database.
- `codex-threads --json messages search "build a CLI" --limit 20`
  Searches cleaned message text, not raw tool-output noise.
- `codex-threads --json threads resolve "tweet idea" --limit 20`
  Finds likely matching threads by project/cwd/path matches first, then message hits.
- `codex-threads --json threads recent --limit 20 --cwd /Users/.../workspace/opensource`
  Lists the most recent threads, with optional project or cwd filtering.
- `codex-threads --json threads read <session-id>`
  Reads the cleaned conversation for one thread and collapses large pasted skill/context blocks by default.
- `codex-threads --json events read <session-id> --limit 50`
  Reads the recent event stream for one thread.
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
codex-threads --json threads recent --limit 10 --project opensource
codex-threads --json messages search "build a CLI" --limit 20
codex-threads --json threads resolve "tweet idea" --limit 20
codex-threads --json threads read 019d8510-9b67-7ff0-914c-cc313085e394
codex-threads --json threads read 019d8510-9b67-7ff0-914c-cc313085e394 --raw
codex-threads --json events read 019d8510-9b67-7ff0-914c-cc313085e394 --limit 50
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
- The tool indexes readable `response_item.message` records and keeps the rawer stream for `events read`.
- `threads read` is clean-by-default. Use `--raw` if you want the full stored message text.
- This first version prefers a boring contract over perfect ranking. The important property is that it is deterministic, scriptable, and fast enough to reuse from future Codex threads.
