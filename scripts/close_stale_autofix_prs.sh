#!/usr/bin/env bash
# Close open Copilot Autofix PRs and delete their remote branches.
#
# GitHub Code Quality creates branches like ai-findings-autofix/<path> with draft
# PRs. After landing fixes on main (manually or via merge), run this script to
# avoid stale branches cluttering VS Code and re-triggering CI.
#
# Requires the GitHub CLI (`gh`) with push access to the remote repository.
#
# Usage:
#   ./scripts/close_stale_autofix_prs.sh           # dry-run
#   ./scripts/close_stale_autofix_prs.sh --apply   # close PRs and delete branches
set -euo pipefail

ROOT="$(cd "$(dirname "$0")/.." && pwd)"
cd "$ROOT"

APPLY=0
COMMENT="Fixes already applied on main. Closing stale Copilot Autofix PR."

usage() {
  cat <<'EOF'
Usage: ./scripts/close_stale_autofix_prs.sh [options]

List (default) or close open pull requests whose head branch starts with
ai-findings-autofix/, then delete those remote branches.

Options:
  --apply     Close matching PRs and delete remote branches
  -h, --help  Show this help

Examples:
  ./scripts/close_stale_autofix_prs.sh
  ./scripts/close_stale_autofix_prs.sh --apply
EOF
}

for arg in "$@"; do
  case "$arg" in
    --apply) APPLY=1 ;;
    -h|--help)
      usage
      exit 0
      ;;
    *)
      echo "close_stale_autofix_prs: unknown argument: $arg" >&2
      usage >&2
      exit 2
      ;;
  esac
done

if ! command -v gh >/dev/null 2>&1; then
  echo "close_stale_autofix_prs: gh CLI is required (https://cli.github.com/)" >&2
  exit 2
fi

if ! gh auth status >/dev/null 2>&1; then
  echo "close_stale_autofix_prs: run 'gh auth login' first" >&2
  exit 2
fi

pr_data="$(mktemp "${TMPDIR:-/tmp}/spanda-autofix-prs.XXXXXX")"
trap 'rm -f "$pr_data"' EXIT

gh pr list \
  --state open \
  --limit 100 \
  --json number,title,headRefName \
  --jq '.[] | select(.headRefName | startswith("ai-findings-autofix/")) | "\(.number)\t\(.headRefName)\t\(.title)"' \
  >"$pr_data"

if [[ ! -s "$pr_data" ]]; then
  echo "No open ai-findings-autofix pull requests."
  if [[ "$APPLY" -eq 1 ]]; then
    git fetch --prune origin >/dev/null 2>&1 || true
  fi
  exit 0
fi

branches=()
echo "Open Copilot Autofix pull requests:"
while IFS=$'\t' read -r number branch title; do
  [[ -z "$number" ]] && continue
  branches+=("$branch")
  echo "  - PR #${number} (${branch}): ${title}"
done <"$pr_data"

if [[ "$APPLY" -eq 0 ]]; then
  echo
  echo "Dry run only. Re-run with --apply to close PRs and delete remote branches."
  exit 0
fi

while IFS=$'\t' read -r number branch title; do
  [[ -z "$number" ]] && continue
  gh pr close "$number" --comment "$COMMENT"
done <"$pr_data"

for branch in "${branches[@]}"; do
  git push origin --delete "$branch"
done

git fetch --prune origin
pr_count="$(wc -l <"$pr_data" | tr -d ' ')"
echo "Done. Closed ${pr_count} PR(s) and deleted ${#branches[@]} remote branch(es)."
