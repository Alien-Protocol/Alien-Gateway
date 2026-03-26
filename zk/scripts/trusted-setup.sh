#!/bin/bash

set -e

# ─────────────────────────────────────────────
#  Alien Gateway — Trusted Setup (Linux)
# ─────────────────────────────────────────────

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
ZK_DIR="$(dirname "$SCRIPT_DIR")"

PTAU_DIR="$ZK_DIR/ptau"
BUILD_DIR="$ZK_DIR/build"

CIRCUITS=(
  "merkle_inclusion"
  "merkle_non_inclusion"
  "merkle_update"
  "merkle_update_proof"
  "username_merkle"
  "username_hash"
)

# Power of 2 constraints — merkle_inclusion has ~8070 constraints, needs >= 14
# 2^14 = 16384 — safe for all 3 circuits
POW=14

GREEN="\033[0;32m"
CYAN="\033[0;36m"
RED="\033[0;31m"
RESET="\033[0m"

ok()   { echo -e "${GREEN}  ✔  $1${RESET}"; }
info() { echo -e "${CYAN}▶  $1${RESET}"; }
fail() { echo -e "${RED}  ✘  $1${RESET}"; exit 1; }

echo ""
echo "================================================"
echo "   Alien Gateway — Trusted Setup"
echo "================================================"
echo ""

mkdir -p "$PTAU_DIR"

# ── Phase 1: Powers of Tau ────────────────────

PTAU_0="$PTAU_DIR/pot${POW}_0000.ptau"
PTAU_1="$PTAU_DIR/pot${POW}_0001.ptau"
PTAU_FINAL="$PTAU_DIR/pot${POW}_final.ptau"

if [ ! -f "$PTAU_FINAL" ]; then
  info "Phase 1 — Powers of Tau (bn128, power=$POW)"

  snarkjs powersoftau new bn128 $POW "$PTAU_0" -v
  ok "pot new done"

  snarkjs powersoftau contribute "$PTAU_0" "$PTAU_1" \
    --name="Alien Gateway contribution" -e="$(head -c 64 /dev/urandom | base64)" -v
  ok "pot contribute done"

  snarkjs powersoftau prepare phase2 "$PTAU_1" "$PTAU_FINAL" -v
  ok "pot prepare phase2 done"
else
  ok "Phase 1 already done — skipping ($PTAU_FINAL exists)"
fi

echo ""

# ── Phase 2: Per-circuit setup ────────────────

for CIRCUIT in "${CIRCUITS[@]}"; do
  info "Phase 2 — $CIRCUIT"

  R1CS="$BUILD_DIR/$CIRCUIT/$CIRCUIT.r1cs"
  ZKEY_0="$BUILD_DIR/$CIRCUIT/${CIRCUIT}_0000.zkey"
  ZKEY_FINAL="$BUILD_DIR/$CIRCUIT/${CIRCUIT}_final.zkey"
  VKEY="$BUILD_DIR/$CIRCUIT/verification_key.json"

  [ -f "$R1CS" ] || fail "$CIRCUIT.r1cs not found — run compile first"

  snarkjs groth16 setup "$R1CS" "$PTAU_FINAL" "$ZKEY_0"
  ok "$CIRCUIT groth16 setup done"

  snarkjs zkey contribute "$ZKEY_0" "$ZKEY_FINAL" \
    --name="$CIRCUIT contribution" -e="$(head -c 64 /dev/urandom | base64)" -v
  ok "$CIRCUIT zkey contribute done"

  snarkjs zkey export verificationkey "$ZKEY_FINAL" "$VKEY"
  ok "$CIRCUIT verification key exported"

  echo "     ├── $ZKEY_FINAL"
  echo "     └── $VKEY"
  echo ""
done

echo "================================================"
echo -e "${GREEN}   Trusted setup complete!${RESET}"
echo "================================================"
echo ""