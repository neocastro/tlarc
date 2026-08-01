\* Feature class: LET — local operator definitions in an expression.
\*
\* Core TLA+ only (no EXTENDS): `Next` uses `LET step(v) == ... IN ...` to
\* compute the successor; `x` climbs 0 -> 1 -> 2 and stays, so the
\* reachable state space is finite.
---- MODULE letin ----
VARIABLE x

Init == x = 0

Next == LET step(v) == IF v = 0 THEN 1 ELSE 2
        IN  x' = step(x)

Inv == x \in {0, 1, 2}

====
