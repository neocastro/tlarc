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
