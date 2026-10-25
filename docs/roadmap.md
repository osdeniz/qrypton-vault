# Roadmap

Milestones for the Stellar Community Fund Instawards application. Each milestone ends in a public, verifiable deliverable in this repository.

## Dependency

M1 depends on Protocol 30 (which includes [CAP-0087](https://github.com/stellar/stellar-protocol/blob/master/core/cap-0087.md), native ML-DSA verification) being enabled on testnet, and on a stable `soroban-sdk` / `stellar-cli` 30.x release. Until then, contract logic is developed and tested against `MockVerifier`.

## M1 — Vault contract and native ML-DSA verification

- Implement the vault contract: constructor, deposit, withdraw (expiry → nonce → policy → M-of-N → transfer → event), views.
- Implement `NativeHostVerifier` on the CAP-0087 host functions.
- Implement the canonical message encoder on-chain and off-chain, with shared test vectors.

**Done when:** an ML-DSA-65-approved withdrawal executes on testnet, and `docs/benchmarks.md` reports measured CPU instructions, memory and fees for 1-of-1, 2-of-3 and 3-of-5 approvals (ML-DSA-65 and ML-DSA-87).

## M2 — Signer, software keystore, M-of-N approval

- `qrypton-signer` CLI: `keygen`, `show-pubkey`, `approve-withdraw`.
- `SoftwareKeystore` backend (encrypted at rest; development and testnet only), using an audited ML-DSA library.
- Approval flow: decode and display → policy pre-check → sign → approval.
- Governance flows: `set_policy`, `propose_key_rotation`, `execute_key_rotation`, `cancel_key_rotation`.

**Done when:** a 2-of-3 testnet withdrawal succeeds with three independent approvers, and tests cover duplicate signers, wrong network, wrong vault, expired request, reused nonce, algorithm mismatch and policy violations.

## M3 — SDK and anchor demo

- `qrypton-sdk`: `build_withdraw_request`, `collect_approvals`, `submit_withdraw`, `watch_events`.
- The anchor-reserve scenario in `examples/anchor-reserve-withdrawal` running end-to-end.

**Done when:** the scenario runs on testnet, including event-based reconciliation, with a recorded demo and an integration guide published.

## M4 — PKCS#11 HSM and independent security review

- `Pkcs11Hsm` backend: keys generated and used inside an HSM with ML-DSA support.
- Full threat model.
- Independent security review of the contract and signer.

**Done when:** an HSM-signed approval executes on testnet, and the review report and the resulting fixes are published.
