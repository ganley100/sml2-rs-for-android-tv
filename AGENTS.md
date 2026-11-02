# AGENTS.md — sml2-rs

Native Rust port of Super Mario Land 2, verified against `../sml2decomp/` and `baserom.gb`.

## Read this first

| Doc | Use |
|-----|-----|
| [`ORCHESTRATION.md`](ORCHESTRATION.md) | **Supervisor loop**, worker prompts, model roles, accept/reject |
| [`PORTING.md`](PORTING.md) | Verification, one-ticket rules, forbidden moves |
| [`docs/idiomatic-rust-porting.md`](docs/idiomatic-rust-porting.md) | Typed native actors, shared domain behavior, slot conversion boundary |
| [`docs/route-matrix.md`](docs/route-matrix.md) | Completion evidence and remaining route/integration work |
| [`DESIGN.md`](DESIGN.md) | Architecture, renderer seams, standing goals |
| [`PORT_QUEUE.json`](PORT_QUEUE.json) | **Pending items only** — `python3 -c 'import json;print(json.load(open("PORT_QUEUE.json"))["queue"])'` |
| [`PORT_QUEUE.done.json`](PORT_QUEUE.done.json) | Archived completed items (ignore unless auditing history) |
| [`captures/README.md`](captures/README.md) | Frozen replay corpus (read-only) |
| [`docs/agent-prompts.md`](docs/agent-prompts.md) | Copy-paste supervisor and Porter system prompts |

## Quick gate

```bash
cargo test -p sml2-core
set -o pipefail
bash captures/sweep.sh 2>&1 | tail -n 25   # once per work package, after tests pass
```

## Large files

Do not load into context: `captures/*.json`, `PORT_QUEUE.done.json`, whole `DESIGN.md` for a handler task.

```bash
python3 -c 'import json;print([e["id"] for e in json.load(open("PORT_QUEUE.json"))["queue"]])'
```

## Queue hygiene

- `status` = verification claim, not “lines written”.
- Capture-based gates are void; use `PORTING.md` unit tests.
- Do not append session journals to queue `note` fields — omit them.

## Commits

Create a focused commit after each completed, verified work step. Stage only
files belonging to that step and preserve unrelated local changes.

## Agent skills

### Issue tracker

Issues are tracked in this repository's GitHub Issues. See `docs/agents/issue-tracker.md`.

### Triage labels

Use the default canonical triage labels. See `docs/agents/triage-labels.md`.

### Domain docs

Use the single-context domain-doc layout. See `docs/agents/domain.md`.
