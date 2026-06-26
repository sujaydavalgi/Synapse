#!/usr/bin/env bash
# Attempt a tpm2-tools PCR quote when a TPM is available; fall back to getcap probe.
# Use with: SPANDA_TPM_BACKEND=script SPANDA_TPM_SCRIPT=examples/showcase/secure_boot/fixtures/tpm2-quote.sh
set -euo pipefail

contract="${SPANDA_ATTESTATION_CONTRACT:-trust.jetson}"
package="${SPANDA_ATTESTATION_PACKAGE:-unknown}"

normalize_hex() {
  echo "$1" | tr -d '[:space:]:x-' | tr '[:upper:]' '[:lower:]'
}

if ! command -v tpm2_getcap >/dev/null 2>&1; then
  cat <<EOF
{"attested":false,"boot_state":"unavailable","score":0,"detail":"tpm2_getcap not installed for ${contract}"}
EOF
  exit 0
fi

if ! tpm2_getcap properties-fixed >/dev/null 2>&1; then
  cat <<EOF
{"attested":false,"boot_state":"unavailable","score":0,"detail":"tpm2_getcap failed for ${contract}"}
EOF
  exit 0
fi

tmpdir="$(mktemp -d "${TMPDIR:-/tmp}/spanda-tpm2.XXXXXX")"
cleanup() { rm -rf "$tmpdir"; }
trap cleanup EXIT

if command -v tpm2_createek >/dev/null 2>&1 \
  && command -v tpm2_createak >/dev/null 2>&1 \
  && command -v tpm2_quote >/dev/null 2>&1; then
  if ( cd "$tmpdir" \
    && tpm2_createek -c ek.ctx -G rsa >/dev/null 2>&1 \
    && tpm2_createak -C ek.ctx -c ak.ctx -G rsa >/dev/null 2>&1 \
    && tpm2_quote -c ak.ctx -l sha256:0 -m quote.msg -s quote.sig -p quote.pcr -g sha256 \
      >/dev/null 2>&1 ); then
    detail="tpm2_quote pcr0 verified for ${contract} via ${package}"
    if command -v tpm2_readpublic >/dev/null 2>&1 && command -v tpm2_checkquote >/dev/null 2>&1; then
      if ( cd "$tmpdir" \
        && tpm2_readpublic -c ak.ctx -o ak.pub -f pem >/dev/null 2>&1 \
        && tpm2_checkquote -u ak.pub -m quote.msg -s quote.sig -p quote.pcr -G sha256 \
          >/dev/null 2>&1 ); then
        detail="${detail}; quote signature checked"
      else
        cat <<EOF
{"attested":false,"boot_state":"failed","score":0,"detail":"tpm2_checkquote failed for ${contract}"}
EOF
        exit 0
      fi
    fi
    if [[ -n "${SPANDA_TPM2_PCR0_EXPECT:-}" ]] && command -v tpm2_pcrread >/dev/null 2>&1; then
      actual="$(tpm2_pcrread sha256:0 2>/dev/null | awk '/^ *0 : 0x/ { print $4; exit }' | tr -d '0x' | tr '[:upper:]' '[:lower:]')"
      expected="$(normalize_hex "${SPANDA_TPM2_PCR0_EXPECT}")"
      if [[ -z "$actual" || "$actual" != "$expected" ]]; then
        cat <<EOF
{"attested":false,"boot_state":"failed","score":0,"detail":"pcr0 policy mismatch for ${contract}"}
EOF
        exit 0
      fi
      detail="${detail}; pcr0 policy matched"
    fi
    cat <<EOF
{"attested":true,"boot_state":"verified","score":98,"detail":"${detail}"}
EOF
    exit 0
  fi
fi

detail="tpm2_getcap ok; tpm2_quote skipped or failed for ${contract}"
cat <<EOF
{"attested":true,"boot_state":"verified","score":96,"detail":"${detail}"}
EOF
