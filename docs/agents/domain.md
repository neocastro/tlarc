# Domain Docs

Single context. The canonical glossary is `CONTEXT.md` at the repo root;
architectural decisions live in `docs/adr/` (ADR-0001 differential testing,
ADR-0002 SANY bridge).

Consumers:

- Read `CONTEXT.md` before proposing terms, writing issues, or naming
  things — use the glossary, flag ambiguities there.
- Respect the ADRs when touching the areas they govern: the differential
  harness is the acceptance authority (0001); the Java bridge and its
  `tla-ast/v1` schema are deliberate (0002).
- The cross-language AST contract is documented in `docs/ast-schema.md`;
  bump schema, bridge, and doc together.

## Reference resources

Use these while porting TLC — consult them before guessing at behavior:

- **DeepWiki: tlaplus/tlaplus** — https://deepwiki.com/tlaplus/tlaplus —
  AI-generated deep-dive of the reference Java codebase (TLC, SANY, and
  friends). Good for tracing how the reference implementation actually works.
- **TLA+ docs wiki** — https://docs.tlapl.us/start — community documentation
  hub for TLA+ and the tools. Good for language semantics and tool behavior
  written up as prose rather than Java source.
- **Local tlaplus source checkout** — `/home/neo/dev/algebraik/tlaplus` —
  shallow clone of `tlaplus/tlaplus` at tag `v1.7.4` (matches the pinned
  `tla2tools.jar`). TLC/SANY sources under
  `tlatools/org.lamport.tlatools/src/` (e.g. `tlc2/TLC.java`). Use `rg` here
  instead of fetching Java source from the web; the version matches what the
  differential harness runs.
