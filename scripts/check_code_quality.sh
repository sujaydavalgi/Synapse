#!/usr/bin/env bash
# List open GitHub Code Quality standard (CodeQL) findings for this repository.
#
# Requires the GitHub CLI (`gh`) with access to the remote repository.
# AI findings are not available via the REST API; review those in GitHub under
# Security and quality -> AI findings after pushing to the default branch.
#
# Usage:
#   ./scripts/check_code_quality.sh
#   CODE_QUALITY_STATE=open ./scripts/check_code_quality.sh
#   CODE_QUALITY_WARN_ONLY=1 ./scripts/check_code_quality.sh
set -euo pipefail

ROOT="$(cd "$(dirname "$0")/.." && pwd)"
cd "$ROOT"

API_VERSION="2026-03-10"
STATE="${CODE_QUALITY_STATE:-open}"
WARN_ONLY="${CODE_QUALITY_WARN_ONLY:-0}"

usage() {
  cat <<'EOF'
Usage: ./scripts/check_code_quality.sh [options]

Query GitHub Code Quality standard findings for the current repository.

Options:
  --warn-only   Print findings but exit 0
  -h, --help    Show this help

Environment:
  CODE_QUALITY_STATE      Finding state to query (default: open)
  CODE_QUALITY_WARN_ONLY  Set to 1 to never fail on findings

Examples:
  ./scripts/check_code_quality.sh
  CODE_QUALITY_WARN_ONLY=1 ./scripts/check_code_quality.sh
EOF
}

for arg in "$@"; do
  case "$arg" in
    --warn-only) WARN_ONLY=1 ;;
    -h|--help)
      usage
      exit 0
      ;;
    *)
      echo "check_code_quality: unknown argument: $arg" >&2
      usage >&2
      exit 2
      ;;
  esac
done

if ! command -v gh >/dev/null 2>&1; then
  echo "check_code_quality: gh CLI is required (https://cli.github.com/)" >&2
  exit 2
fi

if ! gh auth status >/dev/null 2>&1; then
  echo "check_code_quality: run 'gh auth login' to query Code Quality findings" >&2
  exit 2
fi

repo_slug="$(gh repo view --json nameWithOwner -q .nameWithOwner)"
findings_file="$(mktemp "${TMPDIR:-/tmp}/spanda-code-quality.XXXXXX")"
trap 'rm -f "$findings_file"' EXIT

gh api -H "X-GitHub-Api-Version: ${API_VERSION}" \
  --paginate \
  "repos/{owner}/{repo}/code-quality/findings?state=${STATE}&per_page=100" \
  --jq '.[] | "\(.location.path):\(.location.start_line) [\(.rule.severity // "note")] \(.rule.id): \(.message.text)"' \
  >"$findings_file"

count=0
if [[ -s "$findings_file" ]]; then
  while IFS= read -r line; do
    [[ -z "$line" ]] && continue
    if [[ "$count" -eq 0 ]]; then
      echo "Code Quality (${STATE}):"
    fi
    count=$((count + 1))
    echo "  - $line"
  done <"$findings_file"
fi

if [[ "$count" -eq 0 ]]; then
  echo "Code Quality (${STATE}): no findings"
  exit 0
fi

echo
echo "Dashboard: https://github.com/${repo_slug}/security/code-quality"
echo "AI findings: https://github.com/${repo_slug}/security (AI findings tab; not available via this script)"
echo "After landing autofixes on main: ./scripts/close_stale_autofix_prs.sh --apply"

if [[ "$WARN_ONLY" == "1" ]]; then
  exit 0
fi

exit 1
