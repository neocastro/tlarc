# tlarc — a Rust reimplementation of TLC

A from-scratch Rust reimplementation of the TLC model checker for TLA+.
Developed incrementally by AI: a strong agent for design/verification, plus a
weak local model (16 GB VRAM GPU) as the bulk implementer grinding small,
machine-verifiable work items. Lives in its own standalone public repo under
the name **tlarc**, aims for community traction.

## Language

**TLA+**:
The temporal logic of actions specification language. The input language we
must parse, analyze, and evaluate. Written in `.tla` files.
_Avoid_: "the spec language", "the DSL"

**SANY**:
The Java reference front-end for TLA+: lexer, parser, semantic analysis, and
level checking. Produces the semantic AST that TLC consumes. **tlarc does not
port SANY** — it calls it as an external Java tool.
_Avoid_: "the parser" (SANY is the whole front-end, not just parsing)

**TLC**:
The Java reference model checker. tlarc reimplements TLC's behavior — the
evaluator, explicit-state enumeration, safety checking — in Rust.
_Avoid_: "the model checker" when referring to the specific reference tool

**Model checking**:
Exhaustively exploring the finite state space of a spec to verify properties.
Distinct from **simulation**, which samples random behaviors.

**State**:
A snapshot of the values of all declared VARIABLEs. The unit of exploration in
model checking.

**Next-state relation**:
The TLA+ action that defines the possible transitions from one state to its
successors. The heart of what a model checker evaluates.

**Differential testing**:
Running tlarc and the reference Java TLC on the same spec + config and
comparing results. The verification spine of the project.
_Avoid_: "testing", "golden tests" when referring to the run-to-run comparison

## Relationships

- A **specification** (.tla file) is parsed and analyzed by **SANY**
- **SANY** runs as an external Java subprocess invoked by tlarc
- **tlarc** evaluates the **next-state relation** to enumerate **states**
- **Differential testing** compares tlarc against the reference **TLC**

## Example dialogue

> **Dev:** "Do we need to port SANY, or can we call the Java one?"
> **Domain expert:** "We call it. The Java shim emits the semantic tree as
> JSON, tlarc consumes it. A native front-end is a possible future milestone,
> not a v1 requirement."

## Flagged ambiguities

- "port" vs "reimplementation" — resolved: **reimplementation of TLC**; SANY
  and PlusCal are not ported. PlusCal is explicitly out of scope forever (the
  user refuses to write PlusCal).
- "model checker" — keep distinct from the reference TLC; the reference is
  always "TLC", ours is "tlarc".
