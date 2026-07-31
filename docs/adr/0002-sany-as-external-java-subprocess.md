# ADR-0002: SANY stays Java — called as an external subprocess

tlarc does not port the TLA+ front-end (SANY: lexer, parser, semantic
analysis, level checking, module resolution — ~53K lines of legacy Java). A
thin Java shim (`sany-json`) runs SANY programmatically against
`tla2tools.jar` and serializes the resolved semantic tree as JSON using a
hand-rolled ~100-line JSON emitter (Gson is only a Maven *build* dependency
of tlatools — it is **not** bundled in the published `tla2tools.jar`, so the
shim stays dependency-free); tlarc deserializes it into Rust AST types via
serde. A JVM is therefore a **runtime** dependency of `tlarc check`, not just
a test dependency.

**Status**: accepted

**Considered options**: full native Rust front-end (largest single work item
in the port, dominates effort for years; weak-model-unfriendly because it is
50K lines of subtle, interdependent parsing/semantics); JNI/embedded JVM
(fast, but fragile, hard for the weak model, and couples the build to JVM
internals); translating the Java parser to Rust mechanically (carries 25
years of accretion and Java idioms into the new codebase).

**Consequences**: the JSON AST schema becomes a stable contract between the
Java and Rust halves — it must be versioned and treated as an API. A future
native parser could replace the bridge without touching the evaluator, as
long as the Rust AST types stay isomorphic to the schema. PlusCal is
permanently out of scope (user decision) — the shim only needs TLA+ front-end
features, never PlusCal translation.
