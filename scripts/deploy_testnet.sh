#!/usr/bin/env bash

set -euo pipefail

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
REPO_ROOT="$(dirname "$SCRIPT_DIR")"
ONCHAIN_DIR="$REPO_ROOT/onchain"
BUILD_DIR="${BUILD_DIR:-$ONCHAIN_DIR/target/deploy}"
OUTPUT_DIR="${OUTPUT_DIR:-$REPO_ROOT/.deployments}"
OUTPUT_FILE="${OUTPUT_FILE:-$OUTPUT_DIR/testnet_contracts.env}"
JSON_OUTPUT_FILE="${JSON_OUTPUT_FILE:-$OUTPUT_DIR/testnet_contracts.json}"
ENV_FILE="${ENV_FILE:-$REPO_ROOT/.env}"

if [[ -f "$ENV_FILE" ]]; then
  set -a
  # shellcheck disable=SC1090
  source "$ENV_FILE"
  set +a
fi

NETWORK="${STELLAR_NETWORK:-${NETWORK:-testnet}}"
SOURCE_ACCOUNT="${STELLAR_ACCOUNT:-${SOURCE_ACCOUNT:-${STELLAR_SECRET_KEY:-${DEPLOYER_SECRET_KEY:-}}}}"
SOURCE_PUBLIC_KEY="${STELLAR_PUBLIC_KEY:-${SOURCE_PUBLIC_KEY:-}}"
DEPLOY_OWNER="${DEPLOY_OWNER:-${OWNER_ADDRESS:-}}"
ESCROW_ADMIN="${ESCROW_ADMIN:-${ADMIN_ADDRESS:-}}"
DEPLOYER_ALIAS="${DEPLOYER_ALIAS:-alien-testnet-deployer}"

CORE_ALIAS="${CORE_ALIAS:-alien-core-testnet}"
ESCROW_ALIAS="${ESCROW_ALIAS:-alien-escrow-testnet}"
FACTORY_ALIAS="${FACTORY_ALIAS:-alien-factory-testnet}"
AUCTION_ALIAS="${AUCTION_ALIAS:-alien-auction-testnet}"

GREEN="\033[0;32m"
YELLOW="\033[1;33m"
CYAN="\033[0;36m"
RESET="\033[0m"

info() {
  echo -e "${CYAN}> $1${RESET}"
}

ok() {
  echo -e "${GREEN}OK${RESET} $1"
}

warn() {
  echo -e "${YELLOW}WARN${RESET} $1"
}

fail() {
  echo "ERROR: $1" >&2
  exit 1
}

if command -v stellar >/dev/null 2>&1; then
  CLI_BIN="${STELLAR_CLI_BIN:-stellar}"
elif command -v soroban >/dev/null 2>&1; then
  CLI_BIN="${STELLAR_CLI_BIN:-soroban}"
else
  fail "Neither 'stellar' nor 'soroban' CLI is installed."
fi

NETWORK_ARGS=()
if [[ -n "${STELLAR_RPC_URL:-}" ]]; then
  NETWORK_ARGS+=(--rpc-url "$STELLAR_RPC_URL")
else
  NETWORK_ARGS+=(--network "$NETWORK")
fi

if [[ -n "${STELLAR_NETWORK_PASSPHRASE:-}" ]]; then
  NETWORK_ARGS+=(--network-passphrase "$STELLAR_NETWORK_PASSPHRASE")
fi

upsert_env() {
  local key="$1"
  local value="$2"
  local tmp_file

  mkdir -p "$(dirname "$ENV_FILE")"
  tmp_file="$(mktemp)"

  if [[ -f "$ENV_FILE" ]]; then
    grep -v -E "^${key}=" "$ENV_FILE" >"$tmp_file" || true
  fi

  printf '%s=%s\n' "$key" "$value" >>"$tmp_file"
  mv "$tmp_file" "$ENV_FILE"
}

resolve_public_key() {
  local account="$1"
  "$CLI_BIN" keys public-key "$account" | tail -n 1 | tr -d '\r'
}

ensure_testnet_source_account() {
  if [[ -n "$SOURCE_ACCOUNT" ]]; then
    return
  fi

  [[ "$NETWORK" == "testnet" ]] || fail "Auto-generating and funding a deployer account is only supported on testnet. Set STELLAR_ACCOUNT in $ENV_FILE for other networks."

  local alias_name="$DEPLOYER_ALIAS"
  local unique_suffix
  local secret_key
  local public_key

  if "$CLI_BIN" keys public-key "$alias_name" >/dev/null 2>&1; then
    unique_suffix="$(date +%s)"
    alias_name="${alias_name}-${unique_suffix}"
  fi

  info "No deployer account found in $ENV_FILE, generating a new testnet identity: $alias_name"
  "$CLI_BIN" keys generate "$alias_name" --overwrite >/dev/null

  info "Funding testnet deployer account"
  "$CLI_BIN" keys fund "${NETWORK_ARGS[@]}" "$alias_name" >/dev/null

  secret_key="$("$CLI_BIN" keys secret "$alias_name" | tail -n 1 | tr -d '\r')"
  public_key="$("$CLI_BIN" keys public-key "$alias_name" | tail -n 1 | tr -d '\r')"

  SOURCE_ACCOUNT="$alias_name"
  SOURCE_PUBLIC_KEY="$public_key"

  upsert_env "STELLAR_NETWORK" "$NETWORK"
  upsert_env "STELLAR_ACCOUNT" "$SOURCE_ACCOUNT"
  upsert_env "STELLAR_PUBLIC_KEY" "$SOURCE_PUBLIC_KEY"
  upsert_env "STELLAR_SECRET_KEY" "$secret_key"
  upsert_env "DEPLOYER_ALIAS" "$alias_name"

  ok "Generated and funded testnet deployer: $public_key"
}

build_contract() {
  local package_name="$1"
  info "Building $package_name"
  "$CLI_BIN" contract build \
    --manifest-path "$ONCHAIN_DIR/Cargo.toml" \
    --package "$package_name" \
    --out-dir "$BUILD_DIR"
}

deploy_contract() {
  local alias_name="$1"
  local wasm_path="$2"

  info "Deploying $(basename "$wasm_path") as alias '$alias_name'"
  "$CLI_BIN" contract deploy \
    "${NETWORK_ARGS[@]}" \
    --source-account "$SOURCE_ACCOUNT" \
    --alias "$alias_name" \
    --wasm "$wasm_path"
}

mkdir -p "$BUILD_DIR" "$OUTPUT_DIR"

ensure_testnet_source_account

if [[ -z "$SOURCE_PUBLIC_KEY" ]]; then
  SOURCE_PUBLIC_KEY="$(resolve_public_key "$SOURCE_ACCOUNT")"
fi

if [[ -z "$DEPLOY_OWNER" ]]; then
  DEPLOY_OWNER="$SOURCE_PUBLIC_KEY"
fi

if [[ -z "$ESCROW_ADMIN" ]]; then
  ESCROW_ADMIN="$SOURCE_PUBLIC_KEY"
fi

upsert_env "STELLAR_NETWORK" "$NETWORK"
upsert_env "STELLAR_ACCOUNT" "$SOURCE_ACCOUNT"
upsert_env "STELLAR_PUBLIC_KEY" "$SOURCE_PUBLIC_KEY"
upsert_env "DEPLOY_OWNER" "$DEPLOY_OWNER"
upsert_env "ESCROW_ADMIN" "$ESCROW_ADMIN"

echo ""
echo "=============================================="
echo " Alien Protocol Testnet Deployment"
echo "=============================================="
echo "CLI:            $CLI_BIN"
echo "Network:        $NETWORK"
echo "Source account: $SOURCE_ACCOUNT"
echo "Public key:     $SOURCE_PUBLIC_KEY"
echo "Core owner:     $DEPLOY_OWNER"
echo "Escrow admin:   $ESCROW_ADMIN"
echo "Env file:       $ENV_FILE"
echo "Build dir:      $BUILD_DIR"
echo "JSON output:    $JSON_OUTPUT_FILE"
echo ""

build_contract "core_contract"
build_contract "escrow_contract"
build_contract "factory_contract"
build_contract "auction_contract"

CORE_WASM="$BUILD_DIR/core_contract.wasm"
ESCROW_WASM="$BUILD_DIR/escrow_contract.wasm"
FACTORY_WASM="$BUILD_DIR/factory_contract.wasm"
AUCTION_WASM="$BUILD_DIR/auction_contract.wasm"

[[ -f "$CORE_WASM" ]] || fail "Missing build artifact: $CORE_WASM"
[[ -f "$ESCROW_WASM" ]] || fail "Missing build artifact: $ESCROW_WASM"
[[ -f "$FACTORY_WASM" ]] || fail "Missing build artifact: $FACTORY_WASM"
[[ -f "$AUCTION_WASM" ]] || fail "Missing build artifact: $AUCTION_WASM"

CORE_ID="$(deploy_contract "$CORE_ALIAS" "$CORE_WASM" | tail -n 1 | tr -d '\r')"
ok "Core deployed: $CORE_ID"

ESCROW_ID="$(deploy_contract "$ESCROW_ALIAS" "$ESCROW_WASM" | tail -n 1 | tr -d '\r')"
ok "Escrow deployed: $ESCROW_ID"

FACTORY_ID="$(deploy_contract "$FACTORY_ALIAS" "$FACTORY_WASM" | tail -n 1 | tr -d '\r')"
ok "Factory deployed: $FACTORY_ID"

AUCTION_ID="$(deploy_contract "$AUCTION_ALIAS" "$AUCTION_WASM" | tail -n 1 | tr -d '\r')"
ok "Auction deployed: $AUCTION_ID"

cat >"$OUTPUT_FILE" <<EOF
STELLAR_NETWORK=$NETWORK
STELLAR_ACCOUNT=$SOURCE_ACCOUNT
STELLAR_PUBLIC_KEY=$SOURCE_PUBLIC_KEY
CORE_CONTRACT_ID=$CORE_ID
ESCROW_CONTRACT_ID=$ESCROW_ID
FACTORY_CONTRACT_ID=$FACTORY_ID
AUCTION_CONTRACT_ID=$AUCTION_ID
CORE_ALIAS=$CORE_ALIAS
ESCROW_ALIAS=$ESCROW_ALIAS
FACTORY_ALIAS=$FACTORY_ALIAS
AUCTION_ALIAS=$AUCTION_ALIAS
EOF

cat >"$JSON_OUTPUT_FILE" <<EOF
{
  "network": "$NETWORK",
  "sourceAccount": "$SOURCE_ACCOUNT",
  "publicKey": "$SOURCE_PUBLIC_KEY",
  "contracts": {
    "core": {
      "id": "$CORE_ID",
      "alias": "$CORE_ALIAS"
    },
    "escrow": {
      "id": "$ESCROW_ID",
      "alias": "$ESCROW_ALIAS"
    },
    "factory": {
      "id": "$FACTORY_ID",
      "alias": "$FACTORY_ALIAS"
    },
    "auction": {
      "id": "$AUCTION_ID",
      "alias": "$AUCTION_ALIAS"
    }
  }
}
EOF

echo ""
echo "Contracts deployed successfully."
echo "Core:    $CORE_ID"
echo "Escrow:  $ESCROW_ID"
echo "Factory: $FACTORY_ID"
echo "Auction: $AUCTION_ID"
echo ""
echo "Saved deployment details to: $OUTPUT_FILE"
echo "Saved deployment JSON to:    $JSON_OUTPUT_FILE"
echo ""
