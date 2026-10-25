# Roadmap

Each milestone ends in a public, verifiable deliverable in this repository.

The first 30-day phase covers M1 and M2 plus the core policy layer (per-withdrawal limit, allowlist, time-locked rotation); M3 and M4 follow.

## Dependency

M1 depends on Protocol 30 reaching testnet (validator vote scheduled for 3 November 2026; Protocol 30 includes [CAP-0087](https://github.com/stellar/stellar-protocol/blob/master/core/cap-0087.md), native ML-DSA verification) and on the release of stellar-cli 30 (with the matching stable `soroban-sdk` 30.x). Until then, contract logic is developed and tested against `MockVerifier`.

## M1 — Vault contract and native ML-DSA verification

- Implement the vault contract: constructor, deposit, withdraw (expiry → nonce → policy → M-of-N → transfer → event), views.
- Core policy layer: per-token per-withdrawal limit, recipient allowlist, pre-allowlisted rescue vault.
- Implement `NativeHostVerifier` on the CAP-0087 host functions.
- Implement the canonical message encoder on-chain and off-chain, with shared test vectors.

**Done when:** an ML-DSA-65-approved withdrawal executes on testnet, and `docs/benchmarks.md` reports measured CPU instructions, memory and fees for 1, 2 and 3 approvals (1-of-1, 2-of-3, 3-of-5) for ML-DSA-44/65/87.

## M2 — Signer, software keystore, M-of-N approval, governance

- `qrypton-signer` CLI: `keygen`, `show-pubkey`, `approve-withdraw`.
- `SoftwareKeystore` backend (encrypted at rest; development and testnet only), using an established open-source Rust ML-DSA library (not yet independently audited; production signing moves to the HSM in M4).
- Approval flow: decode and display → policy pre-check → sign → approval.
- Governance flows: `set_policy` (tightening immediate; loosening time-locked via `execute_policy_change` / `cancel_policy_change`); time-locked key rotation (`propose_key_rotation`, `execute_key_rotation`, `cancel_key_rotation`); time-locked, quorum-only operator change (`propose_operator_change`, `execute_operator_change`, `cancel_operator_change`).

**Done when:** a 2-of-3 testnet withdrawal succeeds with three independent approvers, and tests cover duplicate signers, wrong network, wrong vault, expired request, reused nonce, algorithm mismatch and policy violations, and the governance rules: operator change needs only the quorum and cannot be cancelled by the operator alone, at most one operator change is pending, executing a key rotation cancels a pending operator or policy change, loosening policy changes are time-locked while tightening applies immediately, and rescue-vault withdrawals skip limits.

## M3 — SDK and anchor demo

- `qrypton-sdk`: `build_withdraw_request`, `collect_approvals`, `submit_withdraw`, `watch_events`.
- The anchor-reserve scenario in `examples/anchor-reserve-withdrawal` running end-to-end.

**Done when:** the scenario runs on testnet, including event-based reconciliation, with a recorded demo and an integration guide published.

## M4 — PKCS#11 HSM and independent security review

- `Pkcs11Hsm` backend: keys generated and used inside an HSM with ML-DSA support.
- Full threat model.
- Independent security review of the contract and signer.

**Done when:** an HSM-signed approval executes on testnet, and the review report and the resulting fixes are published.

## Next phase

- Rolling-window (e.g. daily) withdrawal limit. Withdrawals to the rescue vault must be exempt, so a rescue is never blocked by the limit.
