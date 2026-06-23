#!/usr/bin/env bash
# Type-check Spanda examples with manifest-driven expect-fail and skip lists.
set -euo pipefail

ROOT="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
cd "$ROOT"
MANIFEST="$ROOT/scripts/examples-check-manifest.txt"
LIST="$(mktemp)"
trap 'rm -f "$LIST"' EXIT
find examples -name '*.sd' ! -path '*/.spanda/*' ! -name '._*' | sort >"$LIST"

if [[ -n "${SPANDA_BIN:-}" ]]; then
  if [[ "$SPANDA_BIN" != /* ]]; then
    SPANDA_BIN="$ROOT/$SPANDA_BIN"
  fi
fi
if [[ -n "${SPANDA_BIN:-}" && -x "${SPANDA_BIN}" ]]; then
  SPANDA=("$SPANDA_BIN")
elif [[ -x "$ROOT/target/release/spanda" ]]; then
  SPANDA=("$ROOT/target/release/spanda")
else
  SPANDA=(cargo run -q -p spanda-cli --)
fi

declare -a CHECKED_PROJECT_PATHS=()
declare -a CHECKED_PROJECT_RCS=()

manifest_has() {
  local kind="$1"
  local file="$2"
  grep -q "^${kind} ${file}$" "$MANIFEST" 2>/dev/null || return 1
}

ensure_project_deps() {
  local pkg_dir="$1"
  if [[ -f "$pkg_dir/spanda.lock" && ! -d "$pkg_dir/.spanda/packages" ]]; then
    (cd "$pkg_dir" && "${SPANDA[@]}" install >/dev/null 2>&1) || return 1
  fi
}

project_check_cached() {
  local pkg_dir="$1"
  local i
  for i in "${!CHECKED_PROJECT_PATHS[@]}"; do
    if [[ "${CHECKED_PROJECT_PATHS[$i]}" == "$pkg_dir" ]]; then
      echo "${CHECKED_PROJECT_RCS[$i]}"
      return 0
    fi
  done
  return 1
}

cache_project_check() {
  CHECKED_PROJECT_PATHS+=("$1")
  CHECKED_PROJECT_RCS+=("$2")
}

check_file() {
  local file="$1"
  local pkg_dir
  pkg_dir="$(dirname "$file")"
  while [[ "$pkg_dir" != "$ROOT/examples" && "$pkg_dir" != "/" && "$pkg_dir" != "." ]]; do
    if [[ -f "$pkg_dir/spanda.toml" ]]; then
      if project_check_cached "$pkg_dir" >/dev/null 2>&1; then
        return "$(project_check_cached "$pkg_dir")"
      fi
      ensure_project_deps "$pkg_dir" || return 1
      local rc=0
      (cd "$pkg_dir" && "${SPANDA[@]}" check --project . >/dev/null 2>&1) || rc=$?
      cache_project_check "$pkg_dir" "$rc"
      return $rc
    fi
    pkg_dir="$(dirname "$pkg_dir")"
  done
  "${SPANDA[@]}" check "$file"
}

total=0
passed=0
failed=0
skipped=0
expected_failed=0

while IFS= read -r file; do
  [[ -z "$file" ]] && continue
  total=$((total + 1))
  if manifest_has skip "$file"; then
    skipped=$((skipped + 1))
    continue
  fi
  if check_file "$file" >/dev/null 2>&1; then
    if manifest_has expect-fail "$file"; then
      echo "UNEXPECTED PASS (expected fail): $file"
      failed=$((failed + 1))
    else
      passed=$((passed + 1))
    fi
  else
    if manifest_has expect-fail "$file"; then
      expected_failed=$((expected_failed + 1))
    else
      echo "FAIL: $file"
      failed=$((failed + 1))
    fi
  fi
done <"$LIST"

echo "Examples: $passed passed, $expected_failed expected-fail, $skipped skipped, $failed unexpected failures (of $total)"

if [[ "$failed" -gt 0 ]]; then
  exit 1
fi
