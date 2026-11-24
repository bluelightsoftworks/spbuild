#!/usr/bin/env bash
#
# rust-checks.sh - run a suite of Rust quality, security and verification tools.
#
# Usage:
#   ./rust-checks.sh                       # run everything
#   ./rust-checks.sh --only clippy,fmt     # run only these steps
#   ./rust-checks.sh --skip kani,rudra     # run everything except these
#   ./rust-checks.sh --list                # list step names
#
# Steps (in order):
#   fmt, clippy, analyzer, audit, deny, auditable, miri, kani, mirai, rudra
#
# Environment variables:
#   LOG_DIR          where per-step logs go              (default: target/checks)
#   NIGHTLY          toolchain used for miri             (default: nightly)
#   MIRAI_TOOLCHAIN  toolchain MIRAI was built with      (default: unset = rustup default)
#   RUDRA_TOOLCHAIN  toolchain Rudra was built with      (default: nightly-2021-10-21)
#   KEEP_GOING       1 = run all steps even if one fails (default: 1)
#
# Exit codes: 0 = all selected steps passed or were skipped, 1 = at least one failed.
#
# Notes:
#   - A missing tool marks the step as SKIP, not FAIL.
#   - MIRAI and Rudra are research tools pinned to old nightly compilers. They
#     frequently break on modern projects, so treat them as best-effort.

set -uo pipefail

LOG_DIR=${LOG_DIR:-target/checks}
NIGHTLY=${NIGHTLY:-nightly}
MIRAI_TOOLCHAIN=${MIRAI_TOOLCHAIN:-}
RUDRA_TOOLCHAIN=${RUDRA_TOOLCHAIN:-nightly-2021-10-21}
KEEP_GOING=${KEEP_GOING:-1}

ALL_STEPS=(fmt clippy analyzer audit deny auditable miri kani mirai rudra)
SKIP_CODE=77

# ---------- helpers ----------------------------------------------------------

have() { command -v "$1" >/dev/null 2>&1; }

# True if `cargo <subcommand>` is installed (optionally under a toolchain: "+nightly")
have_cargo_sub() {
  local tc=""
  if [[ "$1" == +* ]]; then tc=$1; shift; fi
  cargo ${tc:+"$tc"} "$1" --help >/dev/null 2>&1
}

in_list() { # in_list item "a,b,c"
  [[ ",$2," == *",$1,"* ]]
}

usage() { sed -n '2,/^$/p' "$0" | sed 's/^# \{0,1\}//'; }

# ---------- steps ------------------------------------------------------------
# Each step returns 0 (pass), 77 (skip: tool missing) or anything else (fail).

step_fmt() {
  have_cargo_sub fmt || { echo "rustfmt not installed (rustup component add rustfmt)"; return $SKIP_CODE; }
  cargo fmt --all -- --check
}

step_clippy() {
  have_cargo_sub clippy || { echo "clippy not installed (rustup component add clippy)"; return $SKIP_CODE; }
  cargo clippy --workspace --all-targets --all-features -- -D warnings
}

step_analyzer() {
  local ra
  ra=$(command -v rust-analyzer || rustup which rust-analyzer 2>/dev/null) || true
  [[ -n "$ra" ]] || { echo "rust-analyzer not installed (rustup component add rust-analyzer)"; return $SKIP_CODE; }
  # Loads the whole project and reports diagnostics; non-zero exit on errors.
  "$ra" diagnostics .
}

step_audit() {
  have_cargo_sub audit || { echo "cargo-audit not installed (cargo install cargo-audit --locked)"; return $SKIP_CODE; }
  cargo audit --deny warnings
}

step_deny() {
  have_cargo_sub deny || { echo "cargo-deny not installed (cargo install cargo-deny --locked)"; return $SKIP_CODE; }
  cargo deny --all-features check
}

step_auditable() {
  have_cargo_sub auditable || { echo "cargo-auditable not installed (cargo install cargo-auditable --locked)"; return $SKIP_CODE; }

  # Build with the dependencies list embedded in the binaries.
  cargo auditable build --release || return 1

  # Audit the produced binaries, if we can find them.
  if have_cargo_sub audit && have jq; then
    local target_dir bins bin rc=0
    target_dir=$(cargo metadata --no-deps --format-version 1 | jq -r '.target_directory')
    bins=$(cargo metadata --no-deps --format-version 1 \
           | jq -r '.packages[].targets[] | select(.kind | index("bin")) | .name' | sort -u)
    if [[ -z "$bins" ]]; then
      echo "No binary targets found; build succeeded, nothing to audit."
      return 0
    fi
    for bin in $bins; do
      if [[ -f "$target_dir/release/$bin" ]]; then
        echo "--- cargo audit bin $bin"
        cargo audit bin "$target_dir/release/$bin" --deny warnings || rc=1
      fi
    done
    return $rc
  else
    echo "Build OK. Install cargo-audit and jq to also audit the built binaries."
  fi
}

step_miri() {
  have rustup || { echo "rustup is required for miri"; return $SKIP_CODE; }
  have_cargo_sub "+$NIGHTLY" miri || {
    echo "miri not installed (rustup +$NIGHTLY component add miri rust-src)"
    return $SKIP_CODE
  }
  cargo "+$NIGHTLY" miri setup || return 1
  cargo "+$NIGHTLY" miri test
}

step_kani() {
  have_cargo_sub kani || {
    echo "kani not installed (cargo install --locked kani-verifier && cargo kani setup)"
    return $SKIP_CODE
  }
  cargo kani
}

step_mirai() {
  local tc=${MIRAI_TOOLCHAIN:+"+$MIRAI_TOOLCHAIN"}
  have_cargo_sub ${tc:+"$tc"} mirai || {
    echo "cargo-mirai not installed (see https://github.com/facebookexperimental/MIRAI); set MIRAI_TOOLCHAIN if it needs a pinned nightly"
    return $SKIP_CODE
  }
  # Separate target dir so MIRAI's compiler does not clash with the normal build cache.
  CARGO_TARGET_DIR=target/mirai cargo ${tc:+"$tc"} mirai --all-targets
}

step_rudra() {
  have rustup || { echo "rustup is required for rudra"; return $SKIP_CODE; }
  have_cargo_sub "+$RUDRA_TOOLCHAIN" rudra || {
    echo "cargo-rudra not installed for toolchain $RUDRA_TOOLCHAIN (see https://github.com/sslab-gatech/Rudra)"
    return $SKIP_CODE
  }
  local report="$LOG_DIR/rudra-report"
  rm -f "$report"
  CARGO_TARGET_DIR=target/rudra RUDRA_REPORT_PATH="$report" cargo "+$RUDRA_TOOLCHAIN" rudra || return 1

  # Rudra exits 0 even when it finds something, so check the report file.
  if [[ -s "$report" ]]; then
    echo "Rudra reported potential issues:"
    cat "$report"
    return 1
  fi
}

# ---------- argument parsing -------------------------------------------------

ONLY=""
SKIP=""
while [[ $# -gt 0 ]]; do
  case $1 in
    --only) ONLY=${2:-}; shift 2 ;;
    --skip) SKIP=${2:-}; shift 2 ;;
    --list) printf '%s\n' "${ALL_STEPS[@]}"; exit 0 ;;
    -h|--help) usage; exit 0 ;;
    *) echo "Unknown option: $1" >&2; usage >&2; exit 2 ;;
  esac
done

for s in ${ONLY//,/ } ${SKIP//,/ }; do
  in_list "$s" "$(IFS=,; echo "${ALL_STEPS[*]}")" || { echo "Unknown step: $s" >&2; exit 2; }
done

# ---------- main -------------------------------------------------------------

[[ -f Cargo.toml ]] || { echo "Run this from a Cargo project root (no Cargo.toml here)." >&2; exit 2; }
have cargo || { echo "cargo not found in PATH." >&2; exit 2; }

mkdir -p "$LOG_DIR"

declare -A RESULT
ran=()
failed=0

for step in "${ALL_STEPS[@]}"; do
  [[ -n "$ONLY" ]] && ! in_list "$step" "$ONLY" && continue
  [[ -n "$SKIP" ]] && in_list "$step" "$SKIP" && continue

  ran+=("$step")
  log="$LOG_DIR/$step.log"
  printf '==> %-10s ' "$step"

  start=$SECONDS
  "step_$step" >"$log" 2>&1
  rc=$?
  elapsed=$((SECONDS - start))

  case $rc in
    0)         RESULT[$step]="PASS"; echo "PASS (${elapsed}s)" ;;
    $SKIP_CODE) RESULT[$step]="SKIP"; echo "SKIP - $(head -n1 "$log")" ;;
    *)         RESULT[$step]="FAIL"; failed=1
               echo "FAIL (${elapsed}s) - log: $log"
               tail -n 15 "$log" | sed 's/^/      | /' ;;
  esac

  if [[ $rc -ne 0 && $rc -ne $SKIP_CODE && "$KEEP_GOING" != "1" ]]; then
    echo "Stopping early (KEEP_GOING=0)."
    break
  fi
done

echo
echo "================ Summary ================"
for step in "${ran[@]}"; do
  printf '  %-10s %s\n' "$step" "${RESULT[$step]:-NOT RUN}"
done
echo "Logs: $LOG_DIR/"

exit $failed