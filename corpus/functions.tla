\* Feature class: functions and records — function constructor,
\* application, EXCEPT on both, record literal and field access.
\*
\* Core TLA+ only (no EXTENDS): `f` maps {0, 1} to itself and `r` is a
\* two-field record; each step bumps one field to a numeral. The invariant
\* bounds both so the state space stays finite.
---- MODULE functions ----
VARIABLE f, r

Init == f = [i \in {0, 1} |-> i] /\ r = [a |-> 0, b |-> 0]

Next == \/ f' = [f EXCEPT ![0] = 1] /\ r' = r
        \/ r' = [r EXCEPT !.b = 1] /\ f' = f

Inv == f[0] \in {0, 1} /\ f[1] = 1 /\ r.a = 0 /\ r.b \in {0, 1}

====
