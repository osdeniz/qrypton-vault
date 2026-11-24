#!/usr/bin/env bash
# Qrypton Vault — minimal testnet proof.
#
# Deploys the vault with one ML-DSA-65 signer (threshold 1), deposits native
# XLM, and then runs three withdrawals against Stellar testnet (Protocol 30):
#
#   1. a valid ML-DSA-65-approved withdrawal          -> executes
#   2. a valid approval for a request that expires    -> WithdrawRejected event, no funds move
#   3. the same kind of request with a corrupted signature, sent without
#      simulation using the footprint of its valid twin -> fails on-chain
#
# Requirements: stellar-cli 30+, Rust toolchain, curl, python3.
# Stellar identities live in the stellar-cli global config (~/.config/stellar);
# the ML-DSA development keystore and approvals live in $QV_DEMO_DIR (default
# .demo/, git-ignored). Nothing secret is written to the repository.
#
# The software keystore is UNENCRYPTED and for development only.

set -euo pipefail

NETWORK=testnet
PASSPHRASE="Test SDF Network ; September 2015"
RPC_URL="https://soroban-testnet.stellar.org"
EXPLORER="https://stellar.expert/explorer/testnet"
DEMO_DIR="${QV_DEMO_DIR:-.demo}"
OUT="$DEMO_DIR/evidence.env"

cd "$(dirname "$0")/.."
mkdir -p "$DEMO_DIR"
: >"$OUT"

log() { printf '\n== %s\n' "$*" >&2; }
record() { printf '%s=%q\n' "$1" "$2" >>"$OUT"; printf '   %s = %s\n' "$1" "$2" >&2; }
# Extracts the transaction hashes stellar-cli linked in a log file.
tx_hashes() { grep -oE 'tx/[0-9a-f]{64}' "$1" | cut -d/ -f2 | awk '!seen[$0]++'; }

log "Building contract and signer"
stellar contract build --package qrypton-vault-contract >/dev/null
cargo build --quiet --release -p qrypton-signer
WASM=target/wasm32v1-none/release/qrypton_vault_contract.wasm
SIGNER=target/release/qrypton-signer

log "Testnet identities (funded by friendbot)"
for id in qv-operator qv-depositor qv-destination; do
  stellar keys address "$id" >/dev/null 2>&1 || stellar keys generate "$id" --network "$NETWORK" --fund
done
OPERATOR=$(stellar keys address qv-operator)
DEPOSITOR=$(stellar keys address qv-depositor)
DEST=$(stellar keys address qv-destination)
XLM=$(stellar contract id asset --asset native --network "$NETWORK")
record OPERATOR "$OPERATOR"
record DEPOSITOR "$DEPOSITOR"
record DESTINATION "$DEST"
record XLM_SAC "$XLM"

log "ML-DSA-65 approver key (development keystore)"
KEYSTORE="$DEMO_DIR/approver-0.qks"
[ -f "$KEYSTORE" ] || "$SIGNER" keygen --keystore "$KEYSTORE" >/dev/null
PUBKEY_JSON=$("$SIGNER" show-pubkey --keystore "$KEYSTORE" --json)

log "Deploying vault (1 signer, threshold 1, allowlist = destination)"
POLICY="{\"limits\":[],\"window_seconds\":0,\"allowlist\":[\"$DEST\"],\"open_recipients\":false,\"rescue_vault\":null}"
VAULT=$(stellar contract deploy --wasm "$WASM" --source qv-operator --network "$NETWORK" \
  -- --operator "$OPERATOR" --pq_signers "[$PUBKEY_JSON]" --threshold 1 --policy "$POLICY" \
  2>"$DEMO_DIR/deploy.log")
record VAULT "$VAULT"
record WASM_UPLOAD_TX "$(tx_hashes "$DEMO_DIR/deploy.log" | sed -n 1p)"
record CONTRACT_CREATE_TX "$(tx_hashes "$DEMO_DIR/deploy.log" | sed -n 2p)"
record WASM_HASH "$(grep -oE 'wasm hash [0-9a-f]{64}' "$DEMO_DIR/deploy.log" | cut -d' ' -f3)"

log "Depositing 10 XLM"
stellar contract invoke --id "$VAULT" --source qv-depositor --network "$NETWORK" \
  -- deposit --from "$DEPOSITOR" --token "$XLM" --amount 100000000 2>"$DEMO_DIR/deposit.log"
record DEPOSIT_TX "$(tx_hashes "$DEMO_DIR/deposit.log" | tail -1)"

# approve NONCE AMOUNT EXPIRES_AT OUTFILE
approve() {
  "$SIGNER" approve-withdraw --keystore "$KEYSTORE" --signer-index 0 \
    --network-passphrase "$PASSPHRASE" --vault "$VAULT" --to "$DEST" --token "$XLM" \
    --amount "$2" --nonce "$1" --expires-at "$3" --out "$4" --yes 2>/dev/null
}
# request_json NONCE AMOUNT EXPIRES_AT
request_json() {
  printf '{"to":"%s","token":"%s","amount":"%s","nonce":%s,"expires_at":%s}' "$DEST" "$XLM" "$2" "$1" "$3"
}

log "1) Valid ML-DSA-65-approved withdrawal of 2.5 XLM"
EXP1=$(($(date +%s) + 900))
approve 1 25000000 "$EXP1" "$DEMO_DIR/approval-1.json"
ARGS1=(-- withdraw --request "$(request_json 1 25000000 "$EXP1")" --approvals "[$(cat "$DEMO_DIR/approval-1.json")]")
# Simulation metrics for the successful withdrawal.
UNSIGNED=$(stellar contract invoke --id "$VAULT" --source "$OPERATOR" --network "$NETWORK" --build-only "${ARGS1[@]}")
curl -s -X POST "$RPC_URL" -H 'Content-Type: application/json' \
  -d "{\"jsonrpc\":\"2.0\",\"id\":1,\"method\":\"simulateTransaction\",\"params\":{\"transaction\":\"$UNSIGNED\"}}" \
  >"$DEMO_DIR/withdraw-simulation.json"
stellar contract invoke --id "$VAULT" --source qv-operator --network "$NETWORK" "${ARGS1[@]}" \
  >"$DEMO_DIR/withdraw-1.out" 2>"$DEMO_DIR/withdraw-1.log"
record WITHDRAW_TX "$(tx_hashes "$DEMO_DIR/withdraw-1.log" | tail -1)"
record WITHDRAW_RESULT "$(cat "$DEMO_DIR/withdraw-1.out")"

log "2) Approved request that expires before it is submitted"
EXP2=$(($(date +%s) + 20))
approve 2 25000000 "$EXP2" "$DEMO_DIR/approval-2.json"
ARGS2=(-- withdraw --request "$(request_json 2 25000000 "$EXP2")" --approvals "[$(cat "$DEMO_DIR/approval-2.json")]")
echo "   waiting for the ledger clock to pass expires_at=$EXP2 ..." >&2
sleep 45
stellar contract invoke --id "$VAULT" --source qv-operator --network "$NETWORK" "${ARGS2[@]}" \
  >"$DEMO_DIR/withdraw-2.out" 2>"$DEMO_DIR/withdraw-2.log"
record EXPIRED_TX "$(tx_hashes "$DEMO_DIR/withdraw-2.log" | tail -1)"
record EXPIRED_RESULT "$(cat "$DEMO_DIR/withdraw-2.out")"

log "3) Corrupted signature, sent on-chain without simulation"
EXP3=$(($(date +%s) + 900))
approve 3 25000000 "$EXP3" "$DEMO_DIR/approval-3.json"
ARGS3=(-- withdraw --request "$(request_json 3 25000000 "$EXP3")" --approvals "[$(cat "$DEMO_DIR/approval-3.json")]")
# Simulate the VALID twin to obtain footprint and resource fee...
stellar contract invoke --id "$VAULT" --source "$OPERATOR" --network "$NETWORK" --build-only "${ARGS3[@]}" \
  | stellar tx simulate --source-account qv-operator --network "$NETWORK" >"$DEMO_DIR/bad-assembled.xdr"
# ...then flip one byte of the ML-DSA signature (same length, same footprint).
stellar tx decode <"$DEMO_DIR/bad-assembled.xdr" >"$DEMO_DIR/bad-assembled.json"
SIG=$(python3 -c 'import json,sys,re; print(re.search(r"\"MlDsa65\":\"([0-9a-f]+)\"", open(sys.argv[1]).read()).group(1))' "$DEMO_DIR/approval-3.json")
BAD_SIG=$(python3 -c 'import sys; s=sys.argv[1]; b=int(s[200:202],16)^1; print(s[:200]+format(b,"02x")+s[202:])' "$SIG")
python3 - "$DEMO_DIR/bad-assembled.json" "$SIG" "$BAD_SIG" <<'PY'
import sys
path, good, bad = sys.argv[1:]
s = open(path).read()
assert good in s, "signature not found in transaction JSON"
open(path, "w").write(s.replace(good, bad))
PY
# Show what simulation says about the corrupted transaction (it would refuse it).
stellar tx encode <"$DEMO_DIR/bad-assembled.json" | stellar tx simulate --source-account qv-operator --network "$NETWORK" \
  >/dev/null 2>"$DEMO_DIR/bad-simulation.log" || true
stellar tx encode <"$DEMO_DIR/bad-assembled.json" \
  | stellar tx sign --sign-with-key qv-operator --network "$NETWORK" >"$DEMO_DIR/bad-signed.xdr"
record BAD_SIGNATURE_TX "$(stellar tx hash --network "$NETWORK" <"$DEMO_DIR/bad-signed.xdr")"
stellar tx send --network "$NETWORK" <"$DEMO_DIR/bad-signed.xdr" \
  >"$DEMO_DIR/bad-send.json" 2>"$DEMO_DIR/bad-send.log" || true

log "Final balances"
record VAULT_BALANCE "$(stellar contract invoke --id "$XLM" --source qv-operator --network "$NETWORK" --send=no -- balance --id "$VAULT" 2>/dev/null)"
record NONCE_1_USED "$(stellar contract invoke --id "$VAULT" --source qv-operator --network "$NETWORK" --send=no -- is_nonce_used --nonce 1 2>/dev/null)"
record NONCE_2_USED "$(stellar contract invoke --id "$VAULT" --source qv-operator --network "$NETWORK" --send=no -- is_nonce_used --nonce 2 2>/dev/null)"
record NONCE_3_USED "$(stellar contract invoke --id "$VAULT" --source qv-operator --network "$NETWORK" --send=no -- is_nonce_used --nonce 3 2>/dev/null)"

log "Done. Evidence written to $OUT"
echo "   vault: $EXPLORER/contract/$VAULT" >&2
