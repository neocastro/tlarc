# Issue Tracker

Issues live in **GitHub Issues** on this repo (`neocastro/tlarc`).

- CLI: `gh issue create`, `gh issue list`, `gh issue view`
- Work units for AFK agents are single vertical slices, one feature per issue
  (see the `to-issues` skill and `PLAN.md` for the M0–M4 milestones)
- An issue is AFK-ready when it carries the `ready-for-agent` label and its
  body satisfies the acceptance criteria template (see `to-issues`)
- The weak local model grinds `ready-for-agent` issues; the strong agent
  writes them, reviews diffs, and handles HITL items
- Milestone M0 = bootstrap (issues M0.1–M0.7)
