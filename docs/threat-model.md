# Threat model (summary)

A short overview. A full threat model is part of milestone M4 together with the independent security review.

## Assets

- Tokens held by the vault contract.
- Integrity of the PQ signer set, threshold and policy.
- ML-DSA private keys of approvers (in HSMs or, for development, software keystores).
- The classical operator key.
- Integrity of the audit trail (events).

## Adversaries

| Adversary | Capability assumed |
|---|---|
| **Future CRQC operator** | Derives any Ed25519 / ECDSA private key from its public key. Cannot break ML-DSA, SHA-256 or the Soroban host. |
| **External attacker** | Observes all on-chain data, submits arbitrary transactions, replays observed approvals or requests on other networks or vaults. |
| **Compromised insider / approver** | Controls fewer than `threshold` PQ keys and/or the classical operator key. |
| **Compromised operator tooling** | Tampers with requests between the SDK and approvers (e.g. swaps the recipient). |

## Main mitigations

- Two independent authorization layers (classical + M-of-N ML-DSA): a CRQC that breaks the operator key still lacks PQ approvals.
- Canonical, domain-separated messages bound to network and vault; single-use nonces; expiry.
- Approvers decode and display the exact bytes they sign; they never sign opaque blobs.
- Policy limits and allowlist bound the damage of a successful quorum compromise.
- Time-locked, cancellable signer rotation; policy changes require PQ quorum.
- No custom cryptography: ML-DSA verification by the Soroban host, signing by HSM firmware in production (in development, an established open-source Rust ML-DSA library that has not yet been independently audited).

## Out of scope

- Breaks of ML-DSA itself or of the Soroban host implementation.
- Compromise of `threshold` or more approvers together with the classical operator.
- Security of the underlying token contracts and of the Stellar consensus protocol.
- Physical security and operational procedures of the institution's HSMs.
- Side channels in third-party HSM firmware.
- Front-end / UI phishing outside `qrypton-signer`.
- Assets held outside the vault (e.g. in the institution's classic Ed25519 accounts).
