#!/bin/bash

set -e

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
ZK_DIR="$(dirname "$SCRIPT_DIR")"

CIRCUITS_DIR="$ZK_DIR/circuits"
BUILD_DIR="$ZK_DIR/build"
NODE_MODULES="$ZK_DIR/node_modules"

CIRCUITS=(
  "merkle_inclusion|merkle/merkle_inclusion.circom"
  "merkle_non_inclusion|merkle/merkle_non_inclusion.circom"
  "merkle_update|merkle_update.circom"
  "merkle_update_proof|merkle/merkle_update_proof.circom"
  "username_merkle|username_merkle.circom"
  "username_hash|username_hash_main.circom"
)

GREEN="\033[0;32m"
RED="\033[0;31m"
CYAN="\033[0;36m"
RESET="\033[0m"

ok()   { echo -e "${GREEN}  OK  $1${RESET}"; }
fail() { echo -e "${RED}  FAIL  $1${RESET}"; exit 1; }
info() { echo -e "${CYAN}>  $1${RESET}"; }

echo ""
echo "================================================"
echo "   Alien Gateway - ZK Circuit Compiler"
echo "================================================"
echo ""

for entry in "${CIRCUITS[@]}"; do
  NAME="${entry%%|*}"
  CIRCOM_PATH="${entry##*|}"
  SOURCE_BASENAME="$(basename "$CIRCOM_PATH" .circom)"

  info "Compiling: $NAME"

  OUT_DIR="$BUILD_DIR/$NAME"
  WASM_DIR="$OUT_DIR/wasm"

  rm -rf "$OUT_DIR"
  mkdir -p "$OUT_DIR" "$WASM_DIR"

  circom "$CIRCUITS_DIR/$CIRCOM_PATH" \
    --r1cs --sym \
    -o "$OUT_DIR" \
    -l "$NODE_MODULES" \
    || fail "$NAME - r1cs/sym compilation failed"

  circom "$CIRCUITS_DIR/$CIRCOM_PATH" \
    --wasm \
    -o "$WASM_DIR" \
    -l "$NODE_MODULES" \
    || fail "$NAME - wasm compilation failed"

  if [ "$SOURCE_BASENAME" != "$NAME" ]; then
    [ -f "$OUT_DIR/$SOURCE_BASENAME.r1cs" ] && mv "$OUT_DIR/$SOURCE_BASENAME.r1cs" "$OUT_DIR/$NAME.r1cs"
    [ -f "$OUT_DIR/$SOURCE_BASENAME.sym" ] && mv "$OUT_DIR/$SOURCE_BASENAME.sym" "$OUT_DIR/$NAME.sym"
    [ -d "$WASM_DIR/${SOURCE_BASENAME}_js" ] && mv "$WASM_DIR/${SOURCE_BASENAME}_js" "$WASM_DIR/${NAME}_js"
    [ -f "$WASM_DIR/${NAME}_js/${SOURCE_BASENAME}.wasm" ] && mv "$WASM_DIR/${NAME}_js/${SOURCE_BASENAME}.wasm" "$WASM_DIR/${NAME}_js/$NAME.wasm"
  fi

  [ -f "$OUT_DIR/$NAME.r1cs" ] || fail "$NAME - expected normalized r1cs output"
  [ -f "$OUT_DIR/$NAME.sym" ] || fail "$NAME - expected normalized sym output"
  [ -f "$WASM_DIR/${NAME}_js/$NAME.wasm" ] || fail "$NAME - expected normalized wasm output"

  CIRCOM_BASENAME="$(basename "$CIRCOM_PATH" .circom)"
  if [ "$CIRCOM_BASENAME" != "$NAME" ]; then
    # Normalize output names to expected artifact names.
    mv "$OUT_DIR/${CIRCOM_BASENAME}.r1cs" "$OUT_DIR/${NAME}.r1cs"
    mv "$OUT_DIR/${CIRCOM_BASENAME}.sym" "$OUT_DIR/${NAME}.sym"
    mv "$WASM_DIR/${CIRCOM_BASENAME}_js" "$WASM_DIR/${NAME}_js"
    mv "$WASM_DIR/${NAME}_js/${CIRCOM_BASENAME}.wasm" "$WASM_DIR/${NAME}_js/${NAME}.wasm"
  fi

  ok "$NAME compiled"
  echo "     |- $OUT_DIR/$NAME.r1cs"
  echo "     |- $OUT_DIR/$NAME.sym"
  echo "     '- $WASM_DIR/${NAME}_js/$NAME.wasm"
  echo ""
done

echo "================================================"
echo -e "${GREEN}   All circuits compiled successfully!${RESET}"
echo "================================================"
echo ""
