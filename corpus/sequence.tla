\* Feature class: sequencing of assignments — a multi-variable action
\* that advances a state machine through ordered phases.
\*
\* Core TLA+ only (no EXTENDS): `x` advances 0 -> 1 -> 2 -> 3; `y` lags
\* one phase behind, changing only when `x` reaches a new milestone. The
\* guarded disjunction makes the order explicit and the state space finite.
---- MODULE sequence ----
VARIABLE x, y

Init == x = 0 /\ y = 0

Next == \/ x = 0 /\ x' = 1 /\ y' = y
        \/ x = 1 /\ x' = 2 /\ y' = 1
        \/ x = 2 /\ x' = 3 /\ y' = 2
        \/ x = 3 /\ x' = 3 /\ y' = 2

Inv == x \in {0, 1, 2, 3} /\ y \in {0, 1, 2}

====
