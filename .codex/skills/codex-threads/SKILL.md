---
name: codex-threads
description: Search and read local Codex session archives through the codex-threads CLI.
---

# codex-threads

Use this when you need to find an old Codex thread, resolve the right session id, or read a cleaned thread without re-feeding raw archive noise.

## Verify the command

```bash
command -v codex-threads
codex-threads --json doctor
```

## Safe order of operations

1. Refresh the local index first.

```bash
codex-threads --json sync
```

2. Search messages when you know a phrase from the thread.

```bash
codex-threads --json messages search "build a CLI" --limit 20
```

3. Resolve a thread when you only know a theme or partial id.

```bash
codex-threads --json threads resolve "tweet idea" --limit 20
```

4. Use `threads recent` when you want the latest work in one repo.

```bash
codex-threads --json threads recent --limit 10 --project opensource
```

5. Read the cleaned thread once you have the session id.

```bash
codex-threads --json threads read <session-id>
```

6. Read recent raw-ish events when you need tool calls, reasoning summaries, or execution trace.

```bash
codex-threads --json events read <session-id> --limit 50
```

## Do not do this without intent

- Do not read raw `.jsonl` files directly unless the CLI output is insufficient.
- Do not assume `threads resolve` is exact; use the returned `session_id` with `threads read`.
- Do not use `threads read --raw` unless you actually want pasted skill blobs and large context blocks.
- Do not parse the human-readable output in automation; use `--json`.
