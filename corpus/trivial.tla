---- MODULE trivial ----
VARIABLE x

Init == x = 0
Next == x' = IF x = 0 THEN 1 ELSE 0
Inv == x \in {0, 1}

====
