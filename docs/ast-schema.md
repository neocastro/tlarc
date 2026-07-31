# AST Schema — `tla-ast/v1`

The contract between the `sany-json` Java bridge (producer) and the tlarc
Rust crate (consumer). This document is authoritative; both sides must stay
in sync with it. Bump `SanyJson.SCHEMA` (Java), `ast::AST_SCHEMA` (Rust), and
this file together.

## Version

`tla-ast/v1` — versioned from day one because the schema is a cross-language
API (see ADR-0002).

## Document

```json
{
  "schema": "tla-ast/v1",
  "module": { ... }
}
```

## Module

```json
{
  "name": "DieHard",
  "constants": [ { "name": "...", "kind": "..." } ],
  "variables": [ { "name": "big", "kind": "..." } ],
  "operators": [ { "name": "Init", "arity": 0, "params": [], "body": { ... } } ]
}
```

- `constants` / `variables`: the module's CONSTANT and VARIABLE declarations.
  `kind` is SANY's printable node-kind name from `ASTConstants.kinds`.
- `operators`: the module's top-level operator definitions.

## Expression nodes

All expressions are objects tagged by `"kind"`:

| kind | fields | notes |
|---|---|---|
| `opappl` | `operator`, `args[]` | Operator application. `operator` is the SANY `SymbolNode` name. |
| `numeral` | `value` | Integer literal as a decimal string (TLA+ integers are unbounded). |
| `string` | `value` | String literal. |
| `letin` | `defs[]`, `body` | `LET d1 == e1 ... dn == en IN body`. `defs` are `OpDef` objects. |
| `unhandled` | `type` | A construct not yet modeled; `type` names the SANY node class, or `"oparg"` for operator arguments. |

## Operator names

`opappl.operator` carries whatever `SymbolNode.getName()` returns:

- User-defined operators: their declared name (e.g. `"Init"`).
- Built-in operators: mostly SANY's internal names, but grammar-symbol
  operators keep their ASCII form. Empirically pinned against
  `tla2tools.jar` v1.7.4 on `corpus/trivial.tla`:

  | TLA+ source | `operator` value |
  |---|---|
  | `=` | `"="` |
  | `'` (prime) | `"'"` |
  | `\in` | `"\\in"` |
  | `IF ... THEN ... ELSE` | `"$IfThenElse"` |
  | `{e1, ..., en}` | `"$SetEnumerate"` |
  | `/\`, `\/` | `"$ConjList"`, `"$DisjList"` (expected; not yet pinned) |
  | `\E`, `\A` | `"$BoundedExists"`, `"$BoundedForall"` (expected; not yet pinned) |

  The M1 evaluator maps these names to evaluation semantics; additions must
  be pinned here and in the bridge integration test as they are implemented.

Unknown operator names must not cause deserialization to fail (the Rust side
ignores unknown fields; the walker emits `unhandled` nodes instead of
crashing).

## Round-trip guarantee

The bridge's integration test (`bridge/`) runs `SanyJson` on
`corpus/trivial.tla` and deserializes the output with the Rust `ast` module.
That test pins the schema's behavior against reality.
