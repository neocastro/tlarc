# Local Agent (the bulk implementer)

The weak local model that grinds `ready-for-agent` issues, per PLAN.md "The
AI workflow". Harness is the **Codewhale TUI runtime**; model is served by
**Ollama** on the local GPU.

## Stack

| Piece | Value |
|---|---|
| Harness | `codewhale` (TUI runtime, `exec` mode) |
| Model server | Ollama at `http://localhost:11434/v1` |
| Model | `gpt-oss:20b` (~14 GB, fits the 16 GB card; tool-capable) |
| Provider id | `ollama` (first-class Codewhale provider) |
| Runner | `scripts/grind-next-issue.sh` |

> Model note: `deepseek-r1:14b` was tried first but cannot call tools in
> Ollama (capabilities: completion+thinking only), and Codewhale `exec
> --auto` requires tool calling. `gpt-oss:20b` supports tools — use it.

## Setup (already applied on this machine)

1. **Ollama model**: `ollama pull gpt-oss:20b`
2. **Codewhale provider** (`~/.codewhale/config.toml`):
   ```
   providers.ollama.base_url = "http://localhost:11434/v1"
   providers.ollama.npm = "@ai-sdk/openai-compatible"
   providers.ollama.model = "gpt-oss:20b"
   ```
   Verified with: `codewhale model resolve --provider ollama gpt-oss:20b`
3. **Minigraf memory MCP** (`~/.codewhale/mcp.json`): `temporal-reasoning`
   server via `uvx --with "mcp<2" temporal-reasoning[git-ingestion]` with
   `MINIGRAF_GRAPH_PATH` set to this repo's `memory.graph`. Without the
   `mcp<2` pin the server crashes (`mcp 2.x` dropped `list_tools`).

## How to run

```
scripts/grind-next-issue.sh            # next ready-for-agent issue
scripts/grind-next-issue.sh --issue 12 # a specific issue
scripts/grind-next-issue.sh --keep-branch  # work on the current branch
scripts/grind-next-issue.sh --solo     # step up: local model is the only agent
```

The runner:
- **Refuses to run on main/master** — the local agent never commits to main.
- Creates `agent/<number>-<slug>` off `origin/main`, unless `--keep-branch`.
- Runs `codewhale --provider ollama exec --auto` with the issue body and the
  working agreements; the agent writes test-first, commits, and opens a PR
  (`gh pr create ... --body "Closes #N"`).

## Solo / fallback mode (`--solo`)

When API credits run out, the strong agent is unavailable and the local model
is the **only agent**. Run with `--solo`: the prompt gains a SOLO OPERATION
block that tells the model to step up into the strong-agent roles:

- Do not wait for a reviewer or human gate — decide and proceed.
- Design the minimal change from the issue's acceptance criteria before coding.
- Self-review the diff against the acceptance criteria before committing.
- Run the full check suite; only open the PR when green.
- If the issue leaves a behavior decision open, pick the option matching the
  repo's working agreements/ADRs and state it in the PR body.

The branch/PR flow and the never-commit-to-main rule are unchanged. The local
model becomes designer + implementer + reviewer in one; the differential
harness (ADR-0001) and CI stay the acceptance authority.

## Division of labor

- **Strong agent** (Claude Code / this harness): writes issues, reviews
  diffs, handles HITL, keeps the graph memory.
- **Local agent** (this setup): one issue at a time, red → green, harness
  decides (ADR-0001 differential testing).
- **Solo mode** (`--solo`): the local agent takes over the strong-agent roles
  too — used when API credits are exhausted (see above).
- The weak model gets small, fully-specified issues only
  (`ready-for-agent` label); never multi-feature tasks.

## Troubleshooting

- `Model error: model 'gpt-oss:20b' not found` → `ollama pull gpt-oss:20b`
- `'Server' object has no attribute 'list_tools'` → the MCP server was
  launched without the `mcp<2` pin (see `docs/agents/memory.md`)
- VRAM contention: Ollama keeps models loaded; if a run OOMs, `ollama stop
  gpt-oss:20b` then retry.
