\* Feature class: basic values — booleans, integers (as numerals), and
\* strings.
\*
\* Core TLA+ only: no EXTENDS, so no arithmetic operators (they live in
\* Naturals). `flag` toggles between the boolean constants, `n` advances
\* between numerals, `tag` moves between two string literals. The
\* invariant pins the value domains so the state space stays finite.
---- MODULE values ----
VARIABLE flag, n, tag

Init == flag = FALSE /\ n = 0 /\ tag = "idle"

Next == \/ (flag = FALSE /\ flag' = TRUE /\ n' = 1 /\ tag' = "running")
        \/ (flag = TRUE /\ flag' = FALSE /\ n' = 2 /\ tag' = "idle")

Inv == flag \in {FALSE, TRUE} /\ n \in {0, 1, 2} /\ tag \in {"idle", "running"}

====
