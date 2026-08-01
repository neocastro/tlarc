# tlarc — Agent Guide

A Rust reimplementation of the TLC model checker for TLA+. See `README.md`
for the project, `PLAN.md` for the phased strategy, `CONTEXT.md` for the
domain glossary.

## Agent skills

This repo uses the standard engineering skill suite. The per-repo
configuration the skills need:

- `docs/agents/issue-tracker.md` — issues live in GitHub Issues on this repo
- `docs/agents/triage-labels.md` — the triage label vocabulary
- `docs/agents/domain.md` — CONTEXT.md + docs/adr/ layout (single context)
- `docs/agents/memory.md` — how to run/query the temporal-reasoning minigraf
  memory (MCP launch needs `mcp<2` pin; direct fallback via the binding)
- `docs/agents/local-agent.md` — the weak local model (Ollama
  `gpt-oss:20b` under `codewhale exec`) that grinds
  `ready-for-agent` issues; run it via `scripts/grind-next-issue.sh`

## Working agreements

- **Work units are GitHub issues** — one vertical slice each, test-first,
  landing green (see `PLAN.md`, "The AI workflow")
- **Acceptance authority**: differential testing against reference Java TLC
  (ADR-0001); a change that regresses the harness does not merge
- **SANY stays Java** — the bridge contract is `tla-ast/v1`
  (ADR-0002, `docs/ast-schema.md`); schema changes bump bridge, Rust, and
  doc together
- **Toolchain**: devbox (`rustup`, `temurin-bin`); `tla2tools.jar` is
  fetched by `scripts/fetch-tla2tools.sh` (pinned, SHA-256 verified)
- **stdout purity**: the bridge's stdout carries only JSON
- **Use `rtk` for shell commands whenever possible** — it filters/summarizes
  output before it hits context (e.g. `rtk ls`, `rtk read`, `rtk git`,
  `rtk gh`, `rtk diff`, `rtk test`, `rtk err`). Saves tokens on every
  command; prefer it over bare `ls`/`cat`/`git`/`gh`/`cargo test` output.
