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

set -Eeuo pipefail
trap 'echo "error: step failed at line $LINENO (see logs in ${QV_DEMO_DIR:-.demo}/)" >&2' ERR

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
die() { printf 'error: %s\n' "$*" >&2; exit 1; }
# Extracts the transaction hashes stellar-cli linked in a log file.
tx_hashes() { grep -oE 'tx/[0-9a-f]{64}' "$1" | cut -d/ -f2 | awk '!seen[$0]++'; }
rpc() {
  curl -s --connect-timeout 10 --max-time 30 -X POST "$RPC_URL" -H 'Content-Type: application/json' \
    -d "{\"jsonrpc\":\"2.0\",\"id\":1,\"method\":\"$1\",\"params\":$2}"
}
# Close time of the latest ledger: the clock the contract checks expiry against.
ledger_time() {
  rpc getLatestLedger '{}' | python3 -c 'import json,sys; print(json.load(sys.stdin)["result"]["closeTime"])'
}
# Status of a submitted transaction (SUCCESS, FAILED or NOT_FOUND), waiting up
# to ~60 s. Transient RPC or parse errors count as "not found yet".
tx_status() {
  local status=NOT_FOUND i
  for i in $(seq 1 12); do
    status=$(rpc getTransaction "{\"hash\":\"$1\"}" \
      | python3 -c 'import json,sys; print(json.load(sys.stdin)["result"]["status"])' 2>/dev/null) \
      || status=NOT_FOUND
    case "$status" in SUCCESS | FAILED) break ;; esac
    sleep 5
  done
  echo "$status"
}

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
# XLM is listed in `limits` so the vault accepts deposits of it (limits are not enforced yet).
POLICY="{\"limits\":[{\"token\":\"$XLM\",\"max_per_withdrawal\":\"1000000000\",\"max_per_window\":\"1000000000\"}],\"window_seconds\":0,\"allowlist\":[\"$DEST\"],\"open_recipients\":false,\"rescue_vault\":null}"
VAULT=$(stellar contract deploy --wasm "$WASM" --source qv-operator --network "$NETWORK" \
  -- --operator "$OPERATOR" --pq_signers "[$PUBKEY_JSON]" --threshold 1 --policy "$POLICY" \
  2>"$DEMO_DIR/deploy.log")
record VAULT "$VAULT"
# The upload transaction only appears when this wasm was not installed before.
if [ "$(tx_hashes "$DEMO_DIR/deploy.log" | wc -l)" -gt 1 ]; then
  record WASM_UPLOAD_TX "$(tx_hashes "$DEMO_DIR/deploy.log" | head -1)"
else
  record WASM_UPLOAD_TX "already-installed"
fi
record CONTRACT_CREATE_TX "$(tx_hashes "$DEMO_DIR/deploy.log" | tail -1)"
record WASM_HASH "$(grep -oE 'wasm hash [0-9a-f]{64}' "$DEMO_DIR/deploy.log" | cut -d' ' -f3)"

log "Depositing 10 XLM"
stellar contract invoke --id "$VAULT" --source qv-depositor --network "$NETWORK" \
  -- deposit --from "$DEPOSITOR" --token "$XLM" --amount 100000000 2>"$DEMO_DIR/deposit.log"
record DEPOSIT_TX "$(tx_hashes "$DEMO_DIR/deposit.log" | tail -1)"

# approve NONCE AMOUNT EXPIRES_AT OUTFILE
approve() {
  "$SIGNER" approve-withdraw --keystore "$KEYSTORE" --signer-index 0 \
    --network-passphrase "$PASSPHRASE" --vault "$VAULT" --to "$DEST" --token "$XLM" \
    --amount "$2" --nonce "$1" --expires-at "$3" --out "$4" --yes 2>"$DEMO_DIR/signer.log" \
    || { cat "$DEMO_DIR/signer.log" >&2; die "qrypton-signer refused to approve nonce $1"; }
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
[ "$(cat "$DEMO_DIR/withdraw-1.out")" = '"Executed"' ] || die "withdrawal 1 did not execute"

log "2) Approved request that expires before it is submitted"
# Expiry must be in the future for both the ledger (checked by the vault) and
# the local clock (checked by the signer before it signs).
NOW_LEDGER=$(ledger_time)
NOW_LOCAL=$(date +%s)
EXP2=$(( (NOW_LEDGER > NOW_LOCAL ? NOW_LEDGER : NOW_LOCAL) + 20 ))
approve 2 25000000 "$EXP2" "$DEMO_DIR/approval-2.json"
ARGS2=(-- withdraw --request "$(request_json 2 25000000 "$EXP2")" --approvals "[$(cat "$DEMO_DIR/approval-2.json")]")
echo "   waiting for the ledger clock to pass expires_at=$EXP2 ..." >&2
for _ in $(seq 1 36); do
  [ "$(ledger_time)" -gt "$EXP2" ] && break
  sleep 5
done
[ "$(ledger_time)" -gt "$EXP2" ] || die "ledger time did not pass expires_at"
stellar contract invoke --id "$VAULT" --source qv-operator --network "$NETWORK" "${ARGS2[@]}" \
  >"$DEMO_DIR/withdraw-2.out" 2>"$DEMO_DIR/withdraw-2.log"
record EXPIRED_TX "$(tx_hashes "$DEMO_DIR/withdraw-2.log" | tail -1)"
record EXPIRED_RESULT "$(cat "$DEMO_DIR/withdraw-2.out")"
[ "$(cat "$DEMO_DIR/withdraw-2.out")" = '{"Rejected":1}' ] || die "expired request was not rejected"

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
BAD_TX=$(stellar tx hash --network "$NETWORK" <"$DEMO_DIR/bad-signed.xdr")
# The send is expected to report a failed transaction; what matters is the
# ledger outcome, checked below.
stellar tx send --network "$NETWORK" <"$DEMO_DIR/bad-signed.xdr" \
  >"$DEMO_DIR/bad-send.json" 2>"$DEMO_DIR/bad-send.log" || true
BAD_STATUS=$(tx_status "$BAD_TX")
[ "$BAD_STATUS" = FAILED ] || { cat "$DEMO_DIR/bad-send.log" >&2; die "corrupted-signature tx is $BAD_STATUS, expected FAILED on-chain"; }
grep -q 'failed verification' "$DEMO_DIR/bad-send.log" && grep -q 'ML_DSA_65' "$DEMO_DIR/bad-send.log" \
  || die "tx failed, but not in the ML-DSA-65 verification"
record BAD_SIGNATURE_TX "$BAD_TX"

log "Final state"
view() { stellar contract invoke --id "$1" --source qv-operator --network "$NETWORK" --send=no -- "${@:2}" 2>/dev/null; }
VAULT_BALANCE=$(view "$XLM" balance --id "$VAULT")
NONCE_1_USED=$(view "$VAULT" is_nonce_used --nonce 1)
NONCE_2_USED=$(view "$VAULT" is_nonce_used --nonce 2)
NONCE_3_USED=$(view "$VAULT" is_nonce_used --nonce 3)
record VAULT_BALANCE "$VAULT_BALANCE"
record NONCE_1_USED "$NONCE_1_USED"
record NONCE_2_USED "$NONCE_2_USED"
record NONCE_3_USED "$NONCE_3_USED"
# 10 XLM deposited, 2.5 XLM withdrawn; nonces 1 and 2 consumed, 3 untouched.
[ "$VAULT_BALANCE" = '"75000000"' ] || die "unexpected vault balance $VAULT_BALANCE"
[ "$NONCE_1_USED $NONCE_2_USED $NONCE_3_USED" = "true true false" ] || die "unexpected nonce state"

log "Done. Evidence written to $OUT"
echo "   vault: $EXPLORER/contract/$VAULT" >&2
