#!/usr/bin/env bash
# Shared SPANDA_REGISTRY_URL helper for smoke scripts and local dev.

ensure_spanda_registry_url() {
  # Set SPANDA_REGISTRY_URL when unset, preferring bundled trust registry.
  #
  # Parameters:
  # - $1 — repository root (required)
  #
  # Returns:
  # 0 when URL is configured.
  #
  # Options:
  # Respects a non-empty existing SPANDA_REGISTRY_URL.

  local repo_root="${1:-}"
  if [[ -z "$repo_root" ]]; then
    echo "ensure_spanda_registry_url: missing repo root" >&2
    return 1
  fi
  if [[ -n "${SPANDA_REGISTRY_URL:-}" ]]; then
    return 0
  fi
  local bundled="${repo_root}/crates/spanda-cli/bundled-registry"
  if [[ -f "${bundled}/index.json" ]]; then
    export SPANDA_REGISTRY_URL="file://${bundled}"
  else
    export SPANDA_REGISTRY_URL="file://${repo_root}/registry"
  fi
}
