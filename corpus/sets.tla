\* Feature class: sets — enumeration, membership, union, difference,
\* and subset.
\*
\* `s` grows and shrinks within a fixed universe {1, 2, 3}; the invariant
\* bounds it with `\subseteq` and the state space stays finite.
---- MODULE sets ----
VARIABLE s

Init == s = {1, 2}

Next == \/ s' = s \cup {3}
        \/ s' = s \ {3}

Inv == s \subseteq {1, 2, 3} /\ 3 \in s => s = {1, 2, 3}

====
