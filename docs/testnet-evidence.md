# Testnet evidence: first ML-DSA-65-approved withdrawal

A minimal proof that the Qrypton Vault withdrawal path works on Stellar testnet (Protocol 30): a vault contract verifies an ML-DSA-65 (NIST FIPS 204) approval with the native CAP-0087 host function, and moves funds only when that approval is valid.

**Scope of this proof.** One ML-DSA-65 signer, threshold 1, one allowlisted recipient, native XLM. Not implemented yet, and not exercised here: token limits, the rescue vault, `set_policy` and all time-locked governance (key rotation, operator change, policy loosening). M-of-N (2-of-3) is covered by local tests only. This is pre-alpha code; do not use it on mainnet.

Reproduce with [`scripts/testnet-demo.sh`](../scripts/testnet-demo.sh). All values below come from a single run of that script on 2026-11-24, recorded with commit `b3fe8f0`. Later hardening (deposits limited to policy-listed tokens, verification stopping at the threshold, a cap on the signer set) does not change the 1-of-1 withdrawal path shown here; the script now also asserts each outcome.

## Setup

| | |
|---|---|
| Network | Stellar testnet, protocol 30 |
| Tooling | `soroban-sdk` 30.0.0, `stellar-cli` 30.0.1 |
| Vault contract | [`CD3NCMOTCBG3QRQEDDCSIPC6R6BOHXXMSK7EBDIKQT6JROSCV3DIH62A`](https://stellar.expert/explorer/testnet/contract/CD3NCMOTCBG3QRQEDDCSIPC6R6BOHXXMSK7EBDIKQT6JROSCV3DIH62A) |
| Wasm hash | `ab47f94f476cce33de66d347944b2136ff5250573604b5b964a5473f92a00b84` |
| Operator (classical factor) | `GCVBYB2CXIKLDNI6NQNMM245WEJL3SHANID2NWCEIY6TGHE3IIFHR2ON` |
| Depositor | `GAYXWUR4CWEBYQNQVWU4WWYI472CSTBNORCRE3WT36N34SNOEEVV5V4W` |
| Allowlisted destination | `GB6X6MBSKHX5YKVCF4CCUT5GRG23TITPS7OAJBVK47IOUTYRW6CIHKXX` |
| Token | native XLM SAC `CDLZFC3SYJYDZT7K67VZ75HPJVIEUVNIXF47ZG2FB2RMQQVU2HHGCYSC` |
| PQ signer | 1 × ML-DSA-65, threshold 1, signed off-chain by `qrypton-signer` (software keystore, `fips204` 0.5.0) |

The approval is produced by the `fips204` crate and verified on-chain by the Soroban host, which uses the RustCrypto `ml-dsa` crate. They are two independent FIPS 204 implementations.

## Transactions

| Step | Result | Transaction |
|---|---|---|
| Upload contract wasm | SUCCESS, ledger 5898331 | [`6b279d25…e86fe6`](https://stellar.expert/explorer/testnet/tx/6b279d25af24bf0f9c069198130df11617c4f6d2504cb278b9d6f7758ae86fe6) |
| Create vault (constructor) | SUCCESS, ledger 5898371 | [`e59edee4…ff2b27`](https://stellar.expert/explorer/testnet/tx/e59edee4e01a08893b9f66972bbb54dac7710315fe7d1f98668cf19ea6ff2b27) |
| Deposit 10 XLM | SUCCESS, ledger 5898372 | [`b03d20c6…83417f`](https://stellar.expert/explorer/testnet/tx/b03d20c631fa17e25b1d679d53ab4036109341a6c5f6878744dce90e2983417f) |
| **Withdraw 2.5 XLM, valid ML-DSA-65 approval** | **SUCCESS**, ledger 5898373, returned `Executed`, `WithdrawExecuted` event | [`885b1c26…90250d`](https://stellar.expert/explorer/testnet/tx/885b1c261846ea11577b38135e8b4b326fb34ac8e52632d528f995387290250d) |
| Withdraw with a valid approval, submitted after `expires_at` | SUCCESS, ledger 5898383, returned `Rejected(Expired)`, `WithdrawRejected` event, **no funds moved** | [`a2eba77b…4dc29f`](https://stellar.expert/explorer/testnet/tx/a2eba77ba76495faac21c32f7ffa96815a19d1198447aba2d28a8579ca4dc29f) |
| Withdraw with a corrupted ML-DSA signature | **FAILED on-chain**, ledger 5898384, host error `failed verification ML_DSA_65` from `verify_sig_ml_dsa_65` | [`eb0eb7fb…ee933d`](https://stellar.expert/explorer/testnet/tx/eb0eb7fb6ca19ede1abc45a2bcf454eec9c727b837b1af90f936e8e16cee933d) |

The wasm had already been installed by an earlier run of the script, so in this run the deploy step only created the contract; the upload transaction above is from that earlier run (same wasm hash). The earlier run's vault, `CBMY2RXM6SQ7HBMGN5VX3U5WHLX5G2IAOAYWBWPBNTEXCVIYLA5QBEYZ`, shows the same three outcomes.

State after the run: vault balance 75,000,000 stroops (7.5 XLM = 10 deposited − 2.5 withdrawn); nonces 1 (executed) and 2 (rejected as expired) are consumed; nonce 3 (corrupted signature) is **not** consumed, because the failed transaction changed nothing.

### How the rejected attempts were shown on-chain

- **Expired request.** The approval was valid, so the quorum check passed; the vault then rejected the request because the ledger time was past `expires_at`. That is a successful transaction with a `WithdrawRejected` event and no transfer. The vault records rejections only after the quorum is verified; see [threat-model.md](threat-model.md).
- **Corrupted signature.** Simulation refuses such a transaction (`Error(Crypto, InvalidInput)`), so it was built from its valid twin: the valid request was simulated to obtain the footprint and resource fee, then one byte of the ML-DSA signature was flipped (same length, so the same footprint), and the transaction was signed and submitted without simulation. The network executed it, the host's ML-DSA-65 verification failed, and the transaction is recorded as failed.

## Cost of the successful withdrawal (1-of-1, ML-DSA-65)

| Metric | Value | Source |
|---|---|---|
| CPU instructions, executed | 3,940,192 | on-chain `core_metrics` (`cpu_insn`) in the transaction meta |
| Memory, executed | 1,409,476 bytes | on-chain `core_metrics` (`mem_byte`) |
| CPU instructions, simulation budget | 4,125,407 | `simulateTransaction` resources (includes the CLI's leeway) |
| Ledger I/O, simulation | 144 bytes read, 468 bytes written | `simulateTransaction` resources |
| Minimum resource fee, simulation | 82,733 stroops | `simulateTransaction` |
| Fee charged | 70,072 stroops (0.0070072 XLM): 100 inclusion + 69,972 resource, 12,761 refunded | transaction result |
| Transaction size | 7,956 bytes | envelope XDR |

Against testnet's per-transaction limits at the time (400,000,000 instructions, 41,943,040 bytes of memory, 132,096 bytes of transaction size), the withdrawal used about 1.0% of the CPU, 3.4% of the memory and 6.0% of the size budget.

The simulation response does not include memory; the memory figure is from the executed transaction. The 2-of-3 and 3-of-5 measurements and the ML-DSA-44/87 comparison are part of milestone M1 and are not done here.

For reference: the expired-request transaction was charged 65,610 stroops in total (100 + 65,510); the failed corrupted-signature transaction 24,038 stroops (100 + 23,938); the deposit 106,319 stroops (100 + 106,219).

## Local checks

`cargo test --workspace` signs approvals with `fips204` and verifies them with the Soroban host's native ML-DSA functions (soroban-sdk testutils, real budget). Covered:

- 1-of-1 and 2-of-3 withdrawals succeed; on-chain message bytes equal the off-chain encoder's output; an independently written test vector pins the format.
- Fail with no state change and no event: corrupted signature, signature for a different amount, approval for another vault or network, no operator authorization, operator without quorum (the nonce stays usable), duplicate approval, unknown signer, algorithm mismatch.
- Replaying the approvals of an executed withdrawal fails with `NonceAlreadyUsed` before any ML-DSA verification (the verification cost counter stays at zero), with no event.
- Quorum-approved but expired, or to a non-allowlisted recipient: `Rejected`, event emitted, nonce consumed, no funds moved.
