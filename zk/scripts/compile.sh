#!/bin/bash

set -e

# ─────────────────────────────────────────────
#  Alien Gateway — ZK Circuit Compiler (Linux)
# ─────────────────────────────────────────────

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
ZK_DIR="$(dirname "$SCRIPT_DIR")"

CIRCUITS_DIR="$ZK_DIR/circuits"
BUILD_DIR="$ZK_DIR/build"
NODE_MODULES="$ZK_DIR/node_modules"

# Circuits to compile: "name|circom_path"
CIRCUITS=(
  "merkle_inclusion|merkle/merkle_inclusion.circom"
  "merkle_update|merkle_update.circom"
  "merkle_update_proof|merkle/merkle_update_proof.circom"
  "username_merkle|username_merkle.circom"
  "username_hash|username_hash.circom"
)

# ── Helpers ───────────────────────────────────

GREEN="\033[0;32m"
RED="\033[0;31m"
CYAN="\033[0;36m"
RESET="\033[0m"

ok()   { echo -e "${GREEN}  ✔  $1${RESET}"; }
fail() { echo -e "${RED}  ✘  $1${RESET}"; exit 1; }
info() { echo -e "${CYAN}▶  $1${RESET}"; }

# ── Main ──────────────────────────────────────

echo ""
echo "================================================"
echo "   Alien Gateway — ZK Circuit Compiler"
echo "================================================"
echo ""

for entry in "${CIRCUITS[@]}"; do
  NAME="${entry%%|*}"
  CIRCOM_PATH="${entry##*|}"

  info "Compiling: $NAME"

  OUT_DIR="$BUILD_DIR/$NAME"
  WASM_DIR="$OUT_DIR/wasm"

  mkdir -p "$OUT_DIR" "$WASM_DIR"

  # Compile r1cs + sym
  circom "$CIRCUITS_DIR/$CIRCOM_PATH" \
    --r1cs --sym \
    -o "$OUT_DIR" \
    -l "$NODE_MODULES" \
    || fail "$NAME — r1cs/sym compilation failed"

  # Compile wasm separately into wasm/ subfolder
  circom "$CIRCUITS_DIR/$CIRCOM_PATH" \
    --wasm \
    -o "$WASM_DIR" \
    -l "$NODE_MODULES" \
    || fail "$NAME — wasm compilation failed"

  ok "$NAME compiled"
  echo "     ├── $OUT_DIR/$NAME.r1cs"
  echo "     ├── $OUT_DIR/$NAME.sym"
  echo "     └── $WASM_DIR/${NAME}_js/$NAME.wasm"
  echo ""
done

echo "================================================"
echo -e "${GREEN}   All circuits compiled successfully!${RESET}"
echo "================================================"
echo ""