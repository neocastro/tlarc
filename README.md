# tlarc

A from-scratch **Rust reimplementation of TLC**, the TLA+ model checker.

tlarc's correctness is defined as behavioral agreement with the reference
Java TLC: the test suite runs both tools on the same spec + config and
requires identical outcomes (see `docs/adr/0001`). The TLA+ front-end (SANY)
is **not** ported — tlarc calls it as an external Java subprocess via a thin
`bridge/` shim (see `docs/adr/0002`).

## Status

Milestone 0 (bootstrap) — the skeleton is under construction. See
[`PLAN.md`](PLAN.md) for the full phased strategy and `CONTEXT.md` for the
project glossary.

## Requirements

- Rust (stable toolchain)
- JDK 11+ at runtime (tlarc invokes the Java SANY front-end)
- `tla2tools.jar` — fetched and verified by `scripts/fetch-tla2tools.sh`

## Quickstart

```sh
# Fetch the pinned, checksum-verified tla2tools.jar
./scripts/fetch-tla2tools.sh

# Build
cargo build

# Run (CLI skeleton at M0)
cargo run -- check --json spec.tla spec.cfg
```

## License

MIT — see [LICENSE](LICENSE). The upstream TLA+ tools this project references
are also MIT-licensed (HP / Microsoft / Linux Foundation).
