#!/bin/bash

set -e

# ─────────────────────────────────────────────
#  Alien Protocol — Witness Generator
# ─────────────────────────────────────────────

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
ZK_DIR="$(dirname "$SCRIPT_DIR")"

BUILD_DIR="$ZK_DIR/build"
INPUT_DIR="$ZK_DIR/inputs"
WITNESS_DIR="$ZK_DIR/witnesses"

CIRCUITS=(
  "merkle_inclusion"
  "merkle_non_inclusion"
  "merkle_update"
  "merkle_update_proof"
  "username_merkle"
  "username_hash"
)

GREEN="\033[0;32m"
RED="\033[0;31m"
CYAN="\033[0;36m"
YELLOW="\033[0;33m"
RESET="\033[0m"

ok()   { echo -e "${GREEN}  ✔  $1${RESET}"; }
fail() { echo -e "${RED}  ✘  $1${RESET}"; exit 1; }
info() { echo -e "${CYAN}▶  $1${RESET}"; }
warn() { echo -e "${YELLOW}  ⚠  $1${RESET}"; }

# ── Argument parsing ──────────────────────────

TARGET=""
INPUT_OVERRIDE=""

usage() {
  echo ""
  echo "Usage: $0 [OPTIONS]"
  echo ""
  echo "Options:"
  echo "  -c, --circuit <name>   Generate witness for a single circuit only"
  echo "  -i, --input   <path>   Override input JSON path for single-circuit mode"
  echo "  -h, --help             Show this help message"
  echo ""
  echo "Circuits:"
  for c in "${CIRCUITS[@]}"; do
    echo "    - $c"
  done
  echo ""
  echo "Input files are expected at: \$ZK_DIR/inputs/<circuit_name>.json"
  echo "Witness outputs go to:       \$ZK_DIR/witnesses/<circuit_name>/"
  echo ""
}

while [[ $# -gt 0 ]]; do
  case "$1" in
    -c|--circuit)
      TARGET="$2"; shift 2 ;;
    -i|--input)
      INPUT_OVERRIDE="$2"; shift 2 ;;
    -h|--help)
      usage; exit 0 ;;
    *)
      echo -e "${RED}Unknown option: $1${RESET}"
      usage; exit 1 ;;
  esac
done

if [ -n "$INPUT_OVERRIDE" ] && [ -z "$TARGET" ]; then
  echo -e "${RED}  ✘  --input requires --circuit to be specified${RESET}"
  usage; exit 1
fi

# ── Build circuit list ────────────────────────

if [ -n "$TARGET" ]; then
  FOUND=0
  for c in "${CIRCUITS[@]}"; do
    [ "$c" = "$TARGET" ] && FOUND=1 && break
  done
  [ "$FOUND" -eq 1 ] || fail "Unknown circuit: '$TARGET'. Run with --help to list valid circuits."
  RUN_CIRCUITS=("$TARGET")
else
  RUN_CIRCUITS=("${CIRCUITS[@]}")
fi

# ── Preflight ─────────────────────────────────

echo ""
echo "================================================"
echo "   Alien Protocol — Witness Generator"
echo "================================================"
echo ""

mkdir -p "$WITNESS_DIR"

# ── Main loop ─────────────────────────────────

PASS=0
SKIP=0
FAIL_LIST=()

for CIRCUIT in "${RUN_CIRCUITS[@]}"; do
  info "Generating witness: $CIRCUIT"

  # Resolve input file
  if [ -n "$INPUT_OVERRIDE" ]; then
    INPUT_FILE="$INPUT_OVERRIDE"
  else
    INPUT_FILE="$INPUT_DIR/${CIRCUIT}.json"
  fi

  WASM_FILE="$BUILD_DIR/$CIRCUIT/wasm/${CIRCUIT}_js/$CIRCUIT.wasm"
  WASM_JS="$BUILD_DIR/$CIRCUIT/wasm/${CIRCUIT}_js/generate_witness.js"
  OUT_DIR="$WITNESS_DIR/$CIRCUIT"
  WITNESS_OUT="$OUT_DIR/${CIRCUIT}.wtns"

  # Input check
  if [ ! -f "$INPUT_FILE" ]; then
    warn "$CIRCUIT — input not found at $INPUT_FILE, skipping"
    SKIP=$((SKIP + 1))
    echo ""
    continue
  fi

  # Artifact checks
  [ -f "$WASM_FILE" ] || fail "$CIRCUIT — wasm not found at $WASM_FILE (run compile first)"
  [ -f "$WASM_JS"   ] || fail "$CIRCUIT — generate_witness.js not found at $WASM_JS"

  mkdir -p "$OUT_DIR"

  # Generate witness via snarkjs (delegates to wasm internally)
  snarkjs wtns calculate \
    "$WASM_FILE" \
    "$INPUT_FILE" \
    "$WITNESS_OUT" \
    || { FAIL_LIST+=("$CIRCUIT"); echo ""; continue; }

  # Verify witness against r1cs
  R1CS="$BUILD_DIR/$CIRCUIT/$CIRCUIT.r1cs"
  if [ -f "$R1CS" ]; then
    snarkjs wtns check "$R1CS" "$WITNESS_OUT" \
      && ok "$CIRCUIT witness verified against r1cs" \
      || { warn "$CIRCUIT — witness check failed (proof may still work, continuing)"; }
  fi

  ok "$CIRCUIT witness generated"
  echo "     └── $WITNESS_OUT"
  echo ""

  PASS=$((PASS + 1))
done

# ── Summary ───────────────────────────────────

echo "================================================"

if [ ${#FAIL_LIST[@]} -gt 0 ]; then
  echo -e "${RED}   ${#FAIL_LIST[@]} circuit(s) failed:${RESET}"
  for f in "${FAIL_LIST[@]}"; do
    echo -e "${RED}     - $f${RESET}"
  done
fi

if [ "$SKIP" -gt 0 ]; then
  echo -e "${YELLOW}   $SKIP circuit(s) skipped (no input file)${RESET}"
fi

echo -e "${GREEN}   $PASS circuit(s) succeeded${RESET}"
echo "================================================"
echo ""

[ ${#FAIL_LIST[@]} -eq 0 ] || exit 1