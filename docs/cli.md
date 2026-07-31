# CLI Contract — `tlarc check`

Stable machine-readable contract for `tlarc check --json <spec.tla> <cfg.tla>`.
Pinned at M0.5; the M0.6 diff harness compares against this shape.

## Verdict (stdout)

With `--json`, stdout carries exactly one JSON object:

```json
{
  "version": "0.0.0",
  "status": "ok",
  "spec": "corpus/trivial.tla",
  "config": "corpus/trivial.cfg",
  "schema": "tla-ast/v1",
  "module": "trivial",
  "counts": { "operators": 3, "variables": 1, "constants": 0 },
  "error": null
}
```

- `status` is `"ok"` when SANY parsed and analyzed the spec, `"parse_error"` otherwise.
- `schema`, `module`, `counts` are present iff `status == "ok"`.
- `error` is present iff `status == "parse_error"` (carries the bridge's
  stderr, i.e. SANY's diagnostics).
- The `.cfg` path is echoed but **not yet interpreted** (M1 work).

## Exit codes

| code | meaning |
|---|---|
| 0 | SANY accepted the spec (`status: "ok"`) |
| 1 | SANY parse/analysis error (`status: "parse_error"`) |
| 2 | CLI usage error (clap) |
| 3+ | bridge unavailable or internal error |

## Streams

- stdout: verdict JSON only (`--json`); human summary otherwise.
- stderr: SANY diagnostics on failure (also carried in `error`), CLI usage
  errors.
