# Testnet evidence: ML-DSA-65-approved withdrawals

A minimal proof that the Qrypton Vault withdrawal path works on Stellar testnet (Protocol 30): a vault contract verifies an ML-DSA-65 (NIST FIPS 204) approval with the native CAP-0087 host function, and moves funds only when that approval is valid.

**Scope of this proof.** One ML-DSA-65 signer, threshold 1, one allowlisted recipient, native XLM. Not implemented yet, and not exercised here: token limit amounts, the rescue vault, `set_policy` and all time-locked governance (key rotation, operator change, policy loosening). M-of-N (2-of-3) is covered by local tests only. This is pre-alpha code; do not use it on mainnet.

Reproduce with [`scripts/testnet-demo.sh`](../scripts/testnet-demo.sh). The script fails unless every on-chain outcome is the expected one: `Executed`, `Rejected(Expired)`, and a corrupted-signature transaction recorded as `FAILED` by the ML-DSA-65 check.

## Latest run: current contract

Run on 2026-11-24 with the contract as of commit `0a67505`. The wasm hash below equals the SHA-256 of `stellar contract build` output for that code (Soroban embeds the contract's doc comments in the wasm, so even documentation changes produce a new hash).

| | |
|---|---|
| Network | Stellar testnet, protocol 30 |
| Tooling | `soroban-sdk` 30.0.0, `stellar-cli` 30.0.1 |
| Vault contract | [`CAOJCV22RYWGIM2OSZXDE4V6CVAYCCVJYCQ6TGXIEZ66DM46VROINYCL`](https://stellar.expert/explorer/testnet/contract/CAOJCV22RYWGIM2OSZXDE4V6CVAYCCVJYCQ6TGXIEZ66DM46VROINYCL) |
| Wasm hash | `64eb89ec6f9c9d5651193332cda6674a1375788a1d6252cb19738f81d0909129` |
| Operator (classical factor) | `GCVBYB2CXIKLDNI6NQNMM245WEJL3SHANID2NWCEIY6TGHE3IIFHR2ON` |
| Depositor | `GAYXWUR4CWEBYQNQVWU4WWYI472CSTBNORCRE3WT36N34SNOEEVV5V4W` |
| Allowlisted destination | `GB6X6MBSKHX5YKVCF4CCUT5GRG23TITPS7OAJBVK47IOUTYRW6CIHKXX` |
| Token | native XLM SAC `CDLZFC3SYJYDZT7K67VZ75HPJVIEUVNIXF47ZG2FB2RMQQVU2HHGCYSC`, listed in the vault policy |
| PQ signer | 1 × ML-DSA-65, threshold 1, signed off-chain by `qrypton-signer` (software keystore, `fips204` 0.5.0) |

The approval is produced by the `fips204` crate and verified on-chain by the Soroban host, which uses the RustCrypto `ml-dsa` crate. They are two independent FIPS 204 implementations.

| Step | Result | Transaction |
|---|---|---|
| Upload contract wasm | SUCCESS, ledger 5900058 | [`997fb165…f87175`](https://stellar.expert/explorer/testnet/tx/997fb16533931a33a7a2a11d9c61b36e661b3d4572ee40161536199e81f87175) |
| Create vault (constructor) | SUCCESS, ledger 5900060 | [`2150f229…59d8f7`](https://stellar.expert/explorer/testnet/tx/2150f229fe4de4b584d5adc7c95bd4ea3c2003d4004c24e18744be6e1859d8f7) |
| Deposit 10 XLM | SUCCESS, ledger 5900062 | [`673b7cb8…2d651f`](https://stellar.expert/explorer/testnet/tx/673b7cb88f199dcc889a619e500df6723bdf97acc4b2dac6aa3de77b332d651f) |
| **Withdraw 2.5 XLM, valid ML-DSA-65 approval** | **SUCCESS**, ledger 5900064, returned `Executed`, `WithdrawExecuted` event | [`60d4fd7f…db98d0`](https://stellar.expert/explorer/testnet/tx/60d4fd7fd6f055641e3db1e4fb47462eefcd291710036dc02a8dd9d93ddb98d0) |
| Withdraw with a valid approval, submitted after `expires_at` | SUCCESS, ledger 5900072, returned `Rejected(Expired)`, `WithdrawRejected` event, **no funds moved** | [`74a64aed…5822c8`](https://stellar.expert/explorer/testnet/tx/74a64aed683d75d4c183308056715b1cd872a40e14feda524d30c41def5822c8) |
| Withdraw with a corrupted ML-DSA signature | **FAILED on-chain**, ledger 5900073, host error `failed verification ML_DSA_65` from `verify_sig_ml_dsa_65` | [`c7cb22bc…c2d060`](https://stellar.expert/explorer/testnet/tx/c7cb22bcfdc34e82cbf9a4f2c010b096fd6b6abc2cf8c8de44ef65dc4ec2d060) |

State after the run, checked by the script: vault balance 75,000,000 stroops (7.5 XLM = 10 deposited − 2.5 withdrawn); nonces 1 (executed) and 2 (rejected as expired) are consumed; nonce 3 (corrupted signature) is **not** consumed, because the failed transaction changed nothing.

### Cost of the successful withdrawal (1-of-1, ML-DSA-65)

| Metric | Value | Source |
|---|---|---|
| CPU instructions, executed | 3,953,585 | on-chain `core_metrics` (`cpu_insn`) in the transaction meta |
| Memory, executed | 1,413,188 bytes | on-chain `core_metrics` (`mem_byte`) |
| CPU instructions, simulation budget | 4,133,876 | `simulateTransaction` resources (includes the CLI's leeway) |
| Ledger I/O, simulation | 144 bytes read, 468 bytes written | `simulateTransaction` resources |
| Minimum resource fee, simulation | 84,988 stroops | `simulateTransaction` |
| Fee charged | 72,034 stroops (0.0072034 XLM): 100 inclusion + 71,934 resource, 13,054 refunded | transaction result |
| Transaction size | 7,956 bytes | envelope XDR |

Against testnet's per-transaction limits at the time (400,000,000 instructions, 41,943,040 bytes of memory, 132,096 bytes of transaction size), the withdrawal used about 1.0% of the CPU, 3.4% of the memory and 6.0% of the size budget.

Of the CPU, one ML-DSA-65 approval accounts for about 2.55 million instructions in the host: 1,505,271 to decode the public key, 47,691 to decode the signature and 994,203 to verify (measured locally with the soroban-sdk cost trackers, which use the host's cost model). The rest is the contract call, storage and the token transfer. The ML-DSA signature is carried twice in the transaction (in the call arguments and in the operator's authorization entry), which is why it is about 8 KB; sending it once is planned for M1.

The simulation response does not include memory; the memory figure is from the executed transaction. The 2-of-3 and 3-of-5 measurements and the ML-DSA-44/87 comparison are part of milestone M1 and are not done here.

For reference: the expired-request transaction was charged 67,573 stroops in total (100 + 67,473); the failed corrupted-signature transaction 24,041 stroops (100 + 23,941); the deposit 110,722 stroops (100 + 110,622). Fees vary slightly between runs because testnet's resource fee rates move with network load.

## First run (commit `b3fe8f0`)

The first recorded run, before the later hardening (deposits limited to policy-listed tokens, quorum checking that stops at the threshold, a cap on the signer set). The 1-of-1 withdrawal path it exercises is the same, and so are the outcomes.

| Step | Result | Transaction |
|---|---|---|
| Vault | wasm `ab47f94f…`, contract [`CD3NCMOT…DIH62A`](https://stellar.expert/explorer/testnet/contract/CD3NCMOTCBG3QRQEDDCSIPC6R6BOHXXMSK7EBDIKQT6JROSCV3DIH62A) | [create `e59edee4…ff2b27`](https://stellar.expert/explorer/testnet/tx/e59edee4e01a08893b9f66972bbb54dac7710315fe7d1f98668cf19ea6ff2b27) |
| Withdraw 2.5 XLM, valid approval | SUCCESS, ledger 5898373, `Executed` | [`885b1c26…90250d`](https://stellar.expert/explorer/testnet/tx/885b1c261846ea11577b38135e8b4b326fb34ac8e52632d528f995387290250d) |
| Valid approval, expired request | SUCCESS, ledger 5898383, `Rejected(Expired)`, no funds moved | [`a2eba77b…4dc29f`](https://stellar.expert/explorer/testnet/tx/a2eba77ba76495faac21c32f7ffa96815a19d1198447aba2d28a8579ca4dc29f) |
| Corrupted signature | FAILED on-chain, ledger 5898384, `failed verification ML_DSA_65` | [`eb0eb7fb…ee933d`](https://stellar.expert/explorer/testnet/tx/eb0eb7fb6ca19ede1abc45a2bcf454eec9c727b837b1af90f936e8e16cee933d) |

Its successful withdrawal used 3,940,192 instructions and 1,409,476 bytes of memory, and was charged 70,072 stroops.

## How the rejected attempts were shown on-chain

- **Expired request.** The approval was valid, so the quorum check passed; the vault then rejected the request because the ledger time was past `expires_at`. That is a successful transaction with a `WithdrawRejected` event and no transfer. The vault records rejections only after the quorum is verified; see [threat-model.md](threat-model.md). The script waits on the latest ledger close time, not the local clock, before submitting.
- **Corrupted signature.** Simulation refuses such a transaction (`Error(Crypto, InvalidInput)`), so it was built from its valid twin: the valid request was simulated to obtain the footprint and resource fee, then one byte of the ML-DSA signature was flipped (same length, so the same footprint), and the transaction was signed and submitted without simulation. The network executed it, the host's ML-DSA-65 verification failed, and the transaction is recorded as failed. The script confirms the `FAILED` status with `getTransaction` before recording the hash.

## Local checks

`cargo test --workspace` signs approvals with `fips204` and verifies them with the Soroban host's native ML-DSA functions (soroban-sdk testutils, real budget). Covered:

- 1-of-1 and 2-of-3 withdrawals succeed; on-chain message bytes equal the off-chain encoder's output; an independently written test vector pins the format.
- Fail with no state change and no event: corrupted signature, signature for a different amount, approval for another vault or network, no operator authorization, operator without quorum (the nonce stays usable), duplicate approval, unknown signer, algorithm mismatch.
- Replaying the approvals of an executed withdrawal fails with `NonceAlreadyUsed` before any ML-DSA verification (the verification cost counter stays at zero), with no event.
- Quorum-approved but expired, or to a non-allowlisted recipient: `Rejected`, event emitted, nonce consumed, no funds moved.
- Approvals are checked in order until M are counted: an invalid approval after the threshold is ignored (exactly M verifications run), one before it fails the call. Deposits of tokens not listed in the policy are refused. The constructor accepts exactly 10 signers and rejects 11, a threshold above the signer count, and duplicate signers.
