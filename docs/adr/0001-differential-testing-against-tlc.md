# ADR-0001: Differential testing against TLC is the acceptance authority

tlarc's correctness is defined as behavioral agreement with the reference Java
TLC. The project runs both on the same `.tla` + `.cfg` and requires identical
outcomes: reachable-state counts, safety verdicts, and counterexample traces.
The tlaplus repo's MIT-licensed `test-model/` corpus is adopted as the seed
regression suite. tlarc therefore deliberately inherits TLC's behavioral
quirks, including its bugs — accepted because the project's value is drop-in
substitutability.

**Status**: accepted

**Considered options**: golden/fixture tests (risk of encoding our own bugs as
truth), property-based tests on the value layer (kept as a complement, not the
authority), hand-written unit tests (minimum bar, circular for semantics).

**Consequences**: a JVM is a permanent build/test dependency; the tlarc CLI
must expose machine-readable output (`--json`) from day one so the diff
harness can compare outcomes without parsing human text; every milestone's
definition of done is "the differential suite stays green on the corpus
exercising the new feature."
