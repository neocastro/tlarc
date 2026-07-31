#!/usr/bin/env bash
# grind-next-issue.sh — hand the next ready-for-agent issue to the local agent.
#
# Uses the Codewhale TUI runtime as the harness and the local Ollama
# deepseek model as the implementer (see docs/agents/local-agent.md).
#
# Flow:
#   1. Refuse to run on main/master (the local agent never commits to main).
#   2. Fetch the oldest open issue labeled `ready-for-agent`.
#   3. Create a feature branch agent/<issue-number>-<slug> off origin/main.
#   4. Run `codewhale --provider ollama exec --auto` with the issue as the
#      task; the agent writes test-first code, commits to the branch, and
#      opens a PR (gh pr create) when the harness is green.
#
# Usage: scripts/grind-next-issue.sh [--issue NUMBER] [--keep-branch]
set -euo pipefail

REPO_ROOT="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
cd "$REPO_ROOT"

ISSUE_NUMBER=""
KEEP_BRANCH=0
while [[ $# -gt 0 ]]; do
  case "$1" in
    --issue) ISSUE_NUMBER="$2"; shift 2 ;;
    --keep-branch) KEEP_BRANCH=1; shift ;;
    *) echo "unknown arg: $1" >&2; exit 2 ;;
  esac
done

# --- 1. Guard: never grind on main ---
CUR_BRANCH="$(git branch --show-current)"
if [[ "$CUR_BRANCH" == "main" || "$CUR_BRANCH" == "master" ]]; then
  echo "refusing to run on ${CUR_BRANCH}: the local agent must never commit to main" >&2
  echo "check out a feature branch (or use --keep-branch to work in-place)" >&2
  exit 1
fi

# --- 2. Pick the issue ---
if [[ -z "$ISSUE_NUMBER" ]]; then
  ISSUE_JSON="$(gh issue list --repo neocastro/tlarc --label ready-for-agent --state open --limit 1 --json number,title,body --jq '.[0]')"
else
  ISSUE_JSON="$(gh issue view "$ISSUE_NUMBER" --repo neocastro/tlarc --json number,title,body --jq '.')"
fi
if [[ -z "$ISSUE_JSON" || "$ISSUE_JSON" == "null" ]]; then
  echo "no ready-for-agent issue found" >&2
  exit 1
fi

NUMBER="$(jq -r .number <<<"$ISSUE_JSON")"
TITLE="$(jq -r .title <<<"$ISSUE_JSON")"
BODY="$(jq -r .body <<<"$ISSUE_JSON")"
echo "==> grinding issue #${NUMBER}: ${TITLE}"

# --- 3. Feature branch (unless working in-place) ---
if [[ "$KEEP_BRANCH" == "0" ]]; then
  SLUG="$(printf '%s' "$TITLE" | tr '[:upper:]' '[:lower:]' | sed -E 's/[^a-z0-9]+/-/g; s/^-+|-+$//g' | cut -c1-40)"
  BRANCH="agent/${NUMBER}-${SLUG}"
  git fetch origin main
  git checkout -b "$BRANCH" origin/main
  echo "==> branch: ${BRANCH}"
fi

# --- 4. Run the local agent ---
PROMPT="You are the tlarc implementer. Work on GitHub issue #${NUMBER}: ${TITLE}

Follow the repo working agreements (CLAUDE.md, docs/agents/): test-first, one vertical slice, differential harness is the acceptance authority.

Issue body:
${BODY}

Rules:
- Work ONLY on the current feature branch; never commit to main.
- Write the failing test first (red), then implement (green).
- Run the project checks (cargo fmt/clippy/test or the differential harness as the issue requires) before committing.
- Commit with a clear message referencing the issue (e.g. \"fix #${NUMBER}: ...\").
- When the work is green, open a PR with: gh pr create --title \"...\" --body \"Closes #${NUMBER}\"
"

codewhale --provider ollama exec --auto "$PROMPT"

echo "==> done grinding issue #${NUMBER}; branch: ${BRANCH:-$(git branch --show-current)}"
