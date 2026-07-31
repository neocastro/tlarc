# tlarc — Gradual Port Strategy (Plan)

Rust reimplementation of the TLC model checker. Decisions from the grilling
session on 2026-07-30; see `CONTEXT.md` for the glossary and
`docs/adr/0001-*`, `docs/adr/0002-*` for the load-bearing choices.

## Locked decisions

- **Product**: CLI-first `tlarc` binary; reimplementation of TLC behavior.
- **Front-end**: SANY stays Java, called as external subprocess via a
  `sany-json` shim (ADR-0002). PlusCal permanently out of scope.
- **Acceptance**: differential testing against reference Java TLC
  (ADR-0001); `test-model/` corpus as seed; MIT license; public repo.
- **Implementer**: weak local model (16 GB VRAM) grinds small verified work
  items; strong agent does design, review, verification.
- **Name**: tlarc.

## Architecture

```
 .tla + .cfg
    │
    ▼
 ┌───────────────────────┐   ┌──────────────────────┐
 │ sany-json (Java shim) │   │ Java TLC (reference) │   ← tla2tools.jar
 │  SANY → JSON AST      │   │  for differential    │     (GitHub release
 └───────────┬───────────┘   │  testing only        │      asset v1.7.4+)
             │ subprocess    └──────────────────────┘
             ▼
 ┌─────────────────────────────────────────────┐
 │ tlarc (Rust)                               │
 │  serde AST types ← JSON schema (versioned) │
 │  Value system (TLA+ values)                │
 │  Evaluator (eval / getNextStates)          │
 │  Model checker (BFS + fingerprints)        │
 │  CLI: tlarc check --json spec.tla cfg.cfg  │
 └─────────────────────────────────────────────┘
             │
             ▼
   diff harness (tlarc vs TLC verdict JSON)
```

Key insight: SANY resolves modules (`EXTENDS`, `INSTANCE`, level checking)
before emitting the tree — module semantics never touch Rust. The Rust side
is evaluation + checking only. `tla2tools.jar` is the single Java dependency
for both the bridge and the diff harness.

## Milestones

### M0 — Bootstrap (skeleton, 1–2 weeks of agent-guided work)

| # | Work item | Done when |
|---|---|---|
| 0.1 | Public `tlarc` repo: MIT LICENSE, README, CONTEXT.md, ADRs, .gitignore, CI (fmt+clippy+test) | CI green on empty crate |
| 0.2 | Fetch `tla2tools.jar` via build script (pinned version, SHA-256 checked) | script reproducible |
| 0.3 | `sany-json` Java shim: run SANY on a spec, emit resolved semantic tree as JSON (Gson) | emits JSON for a trivial spec |
| 0.4 | Rust AST types (serde) + JSON schema document (`docs/ast-schema.md`) | deserializes shim output |
| 0.5 | `tlarc check --json spec.tla cfg.cfg` → prints "N definitions parsed; checking not implemented" + exit code | CLI runs end-to-end |
| 0.6 | Diff harness (`tlarc-test-harness`): runs tlarc + Java TLC, compares JSON verdicts | harness passes on seed corpus |
| 0.7 | Seed corpus (~5 trivial specs, one per later feature class) | harness green |

**Definition of done**: `tlarc check` parses a spec through the bridge and
the harness runs both tools on the corpus.

### M1 — Vertical slice (safety checking on a tiny subset, 1–2 months)

| # | Work item | Done when |
|---|---|---|
| 1.1 | Value system: bool, int, string, tuple, record, set, function (minimal) + `normalize`/`compare` | unit + property tests |
| 1.2 | Fingerprint (64-bit) + in-memory `FPSet` (HashSet of fp) | unit tests, cross-checked against TLC's fp semantics |
| 1.3 | Evaluator subset: literals, vars, set enum, `\in`, union/intersection/difference, arithmetic, `IF/THEN/ELSE`, `LET`, boolean ops | differential green on specs exercising subset |
| 1.4 | `Init`/`Next` handling: conjunction of assignments, `\E x \in S: ...` enumeration | differential green |
| 1.5 | BFS model checker: state queue, workers (single-threaded first), visited-set, invariant + deadlock checks | differential green on ~20 specs |
| 1.6 | `--json` verdict parity: same state count, same invariant/deadlock verdict, equivalent trace | harness parity |
| 1.7 | Trace printing (counterexample output) | diff on traces |

**Definition of done**: tlarc correctly checks specs in the tiny subset —
invariants hold, violations found, deadlocks found — with full parity on the
corpus. This is the "feel the whole loop" milestone.

### M2 — Practical subset (the "useful checker" milestone, 2–4 months)

- EXTENDS standard modules: Naturals, Integers, FiniteSets, Sequences, TLC
- `CHOOSE`, `EXCEPT`, `[x \in S |-> e]`, records, tuples, model values
- cfg features: CONSTANT assignment (incl. model values + symmetry), custom
  Init/Next/invariant names, CONSTRAINT
- `tlarc simulate` (random simulation mode) — low cost, high demo value
- Corpus: curated slice of `test-model/` (~100 specs), differential green

### M3 — Liveness (2–4 months)

- Temporal operators: `[]P`, `<>P`, `[]<>P`, `<>[]P`, `P ~> Q`
- Fairness: `WF_x(A)`, `SF_x(A)`
- Tableau-based liveness checking (Büchi automaton, SCC enumeration)
- Corpus: liveness specs from `test-model/`, differential green

### M4 — Performance & community

- Disk-backed FPSet (mmap), multi-threaded workers, state queue variants
- Statistics output (`--stats`), performance parity targets vs TLC
- Docs, examples, README deep-dive, contribution guide
- Optional future: native Rust front-end replacing the SANY bridge
  (ADR-0002 anticipates this; AST types stay isomorphic to the schema)

## The AI workflow (how the weak model grinds)

1. **Work units are GitHub issues**, one operator/feature per issue, each
   with: the exact TLA+ semantics, the reference TLC behavior (from the Java
   source), example specs, and the acceptance test to write first.
2. **Test-first**: every issue starts red (failing differential or unit
   test), lands green. The strong agent writes the issue; the weak model
   writes the code; the harness decides.
3. **TDD loop**: red → implement → green → refactor. The weak model gets one
   issue at a time; never multi-feature tasks.
4. **Review**: strong agent reviews diffs; CI (fmt, clippy, test,
   differential gate) blocks merges.
5. **Repo setup at M0**: copy the agent-skill structure from this workspace
   (`.codewhale/skills/`, `docs/agents/` with `domain.md`,
   `issue-tracker.md`, `triage-labels.md`) so the same to-issues/triage/tdd
   skills work in the new repo.

## Corpus strategy

- M0: 5 hand-written specs (trivial, one per feature class)
- M1: ~20 specs exercising exactly the M1 subset (legible, debuggable)
- M2: curated slice of `test-model/` (~100), then grow
- M3: liveness corpus
- Every spec is MIT-licensed (upstream) — vendored or referenced, with
  attribution; always keep the corpus runnable by Java TLC too, so the diff
  harness stays the authority.

## Risks & open items

- **TLC quirk inheritance** — accepted (ADR-0001); mismatches in state-count
  corner cases (e.g. set enumeration order) need normalization in the
  harness, not "fixes" in tlarc.
- **JSON schema stability** — the AST schema is a contract; version it from
  day one.
- **Weak-model ceiling** — M3 (liveness) may exceed a 7B model's ability in
  one issue; plan liveness as smaller issues (one tableau node type at a
  time) and keep the strong agent close.
- **JVM as runtime dep** — accepted (ADR-0002); document it prominently in
  the README for community expectations.

## First concrete next steps

1. Create the `tlarc` repo (public) and run M0 items 0.1–0.2.
2. Write the `sany-json` shim (0.3) — requires studying SANY's
   `SpecObj`/`ModuleNode` API in `tla2sany.modanalyzer`.
3. Define the JSON AST schema (0.4) as the first versioned contract.
