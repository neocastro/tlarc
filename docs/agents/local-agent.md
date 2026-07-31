# Local Agent (the bulk implementer)

The weak local model that grinds `ready-for-agent` issues, per PLAN.md "The
AI workflow". Harness is the **Codewhale TUI runtime**; model is served by
**Ollama** on the local GPU.

## Stack

| Piece | Value |
|---|---|
| Harness | `codewhale` (TUI runtime, `exec` mode) |
| Model server | Ollama at `http://localhost:11434/v1` |
| Model | `deepseek-r1:14b` (~9 GB, fits the 16 GB card) |
| Provider id | `ollama` (first-class Codewhale provider) |
| Runner | `scripts/grind-next-issue.sh` |

## Setup (already applied on this machine)

1. **Ollama model**: `ollama pull deepseek-r1:14b`
2. **Codewhale provider** (`~/.codewhale/config.toml`):
   ```
   providers.ollama.base_url = "http://localhost:11434/v1"
   providers.ollama.npm = "@ai-sdk/openai-compatible"
   providers.ollama.model = "deepseek-r1:14b"
   ```
   Verified with: `codewhale model resolve --provider ollama deepseek-r1:14b`
3. **Minigraf memory MCP** (`~/.codewhale/mcp.json`): `temporal-reasoning`
   server via `uvx --with "mcp<2" temporal-reasoning[git-ingestion]` with
   `MINIGRAF_GRAPH_PATH` set to this repo's `memory.graph`. Without the
   `mcp<2` pin the server crashes (`mcp 2.x` dropped `list_tools`).

## How to run

```
scripts/grind-next-issue.sh            # next ready-for-agent issue
scripts/grind-next-issue.sh --issue 12 # a specific issue
scripts/grind-next-issue.sh --keep-branch  # work on the current branch
```

The runner:
- **Refuses to run on main/master** — the local agent never commits to main.
- Creates `agent/<number>-<slug>` off `origin/main`, unless `--keep-branch`.
- Runs `codewhale --provider ollama exec --auto` with the issue body and the
  working agreements; the agent writes test-first, commits, and opens a PR
  (`gh pr create ... --body "Closes #N"`).

## Division of labor

- **Strong agent** (Claude Code / this harness): writes issues, reviews
  diffs, handles HITL, keeps the graph memory.
- **Local agent** (this setup): one issue at a time, red → green, harness
  decides (ADR-0001 differential testing).
- The weak model gets small, fully-specified issues only
  (`ready-for-agent` label); never multi-feature tasks.

## Troubleshooting

- `Model error: model 'deepseek-r1:14b' not found` → `ollama pull deepseek-r1:14b`
- `'Server' object has no attribute 'list_tools'` → the MCP server was
  launched without the `mcp<2` pin (see `docs/agents/memory.md`)
- VRAM contention: Ollama keeps models loaded; if a run OOMs, `ollama stop
  deepseek-r1:14b` then retry (or drop to `deepseek-r1:8b`).
