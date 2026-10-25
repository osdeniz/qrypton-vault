# Threat model (summary)

A short overview. A full threat model is part of milestone M4 together with the independent security review.

## Assets

- Tokens held by the vault contract.
- Integrity of the PQ signer set, threshold, operator and policy (including the rescue vault).
- ML-DSA private keys of approvers (in HSMs or, for development, software keystores).
- The classical operator key.
- Integrity of the audit trail (events).

## Authority model

- The **PQ quorum** (M of N ML-DSA signers) is the root of authority.
- The **operator** (classical Stellar account) is a second factor for withdrawals, policy changes and key rotations.
- A **stolen operator key** alone cannot move funds, change settings or block recovery.
- A **stolen quorum** alone can only replace the operator, and only after a public time lock. During that window the institution can cancel the change and move funds to the pre-allowlisted rescue vault.

## Adversaries

| Adversary | Capability assumed |
|---|---|
| **Future CRQC operator** | Derives any Ed25519 / ECDSA private key from its public key, including the operator key. Cannot break ML-DSA, SHA-256 or the Soroban host. |
| **External attacker** | Observes all on-chain data, submits arbitrary transactions, replays observed approvals or requests on other networks, other vaults or for other actions. |
| **Key thief (operator)** | Holds a copy of, or has destroyed, the classical operator key. |
| **Key thief (quorum)** | Holds copies of `threshold` or more PQ keys, while the institution still holds them too. |
| **Compromised insider / approver** | Controls fewer than `threshold` PQ keys. |
| **Compromised operator tooling** | Tampers with requests between the SDK and approvers (e.g. swaps the recipient). |

## Main mitigations

- PQ quorum required for every outflow and every governance action; a CRQC that breaks the operator key still lacks PQ approvals.
- Operator change needs only the quorum, so a lost or broken classical key cannot lock the vault; the operator cannot cancel it alone, so a stolen operator key cannot block recovery.
- Operator change and key rotation are time-locked and publicly visible; at most one operator change can be pending; executing a key rotation cancels any pending operator change.
- **Pre-allowlisted rescue vault:** withdrawals to it skip amount limits and the allowlist, so a copied quorum is answered by moving funds out within the time lock. The rescue vault is controlled by a separate quorum and operator, preferably on a separate HSM, so the keys an attacker copied do not reach it. Policy changes need the operator, so a quorum-only attacker cannot remove the rescue vault.
- Canonical, domain-separated messages bound to network, vault, action, nonce and expiry; one nonce space for all actions; cancellations bound to a specific proposal.
- Approvers decode and display the exact bytes they sign; they never sign opaque blobs.
- Policy limits and allowlist bound the damage of a compromised quorum plus operator.
- **No upgrade entry point in the current phase:** deployed code cannot be replaced. Migration and incident response use the same path: withdraw to a new (rescue) vault.
- No custom cryptography: ML-DSA verification by the Soroban host, signing by HSM firmware in production (in development, an established open-source Rust ML-DSA library that has not yet been independently audited).

## Residual risks

- A stolen quorum whose theft goes unnoticed for the whole time lock can replace the operator and then control the vault. Monitoring of `OperatorChangeProposed` events is therefore part of operating the vault.
- If the institution has lost (not just leaked) its quorum, the rescue path is unavailable.
- If the rescue vault shares keys or HSMs with the main vault, the rescue path offers no protection against key copying; separation is an operational requirement.
- A quorum-only attacker can re-submit a cancelled operator change indefinitely, so cancelling alone never ends the incident (see below).

## Responding to a stolen quorum

**Key rotation is not a defense once the quorum has been copied.** The attacker's operator change was proposed first, so its time lock expires before any rotation the institution proposes afterwards. Once it executes, the attacker holds both the operator and the quorum and can cancel the pending rotation. Cancelling the attacker's proposal does not help either: it can simply be proposed again.

The correct response is therefore not to race the attacker with cancellations or rotations, but to **move all funds to the rescue vault immediately**, while the institution still holds both the operator and the quorum. Rotation and cancellation are clean-up steps afterwards, not the defense.

## Out of scope

- Breaks of ML-DSA itself or of the Soroban host implementation.
- Simultaneous compromise of the PQ quorum and the operator.
- Security of the underlying token contracts and of the Stellar consensus protocol.
- Physical security and operational procedures of the institution's HSMs.
- Side channels in third-party HSM firmware.
- Front-end / UI phishing outside `qrypton-signer`.
- Assets held outside the vault (e.g. in the institution's classic Ed25519 accounts).
