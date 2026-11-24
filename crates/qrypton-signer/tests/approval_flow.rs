//! Approval-flow tests (software keystore).

use qrypton_signer::approval::{approve_withdraw, ApprovalContext, Confirm};
use qrypton_signer::backend::{KeyBackend, SoftwareKeystore};
use qrypton_signer::SignerError;
use qrypton_types::{AddressBytes, MlDsaLevel, Policy, WithdrawMessage, WithdrawRequest};

struct Answer(bool);

impl Confirm for Answer {
    fn confirm(&self, _: &WithdrawMessage) -> Result<bool, SignerError> {
        Ok(self.0)
    }
}

fn message(expires_at: u64) -> WithdrawMessage {
    WithdrawMessage {
        network_id: [7; 32],
        vault: AddressBytes::from_strkey(
            "CAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAABSC4",
        )
        .unwrap(),
        request: WithdrawRequest {
            to: AddressBytes::from_strkey(
                "GAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAWHF",
            )
            .unwrap(),
            token: AddressBytes::from_strkey(
                "CAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAABSC4",
            )
            .unwrap(),
            amount: 10,
            nonce: 1,
            expires_at,
        },
    }
}

fn ctx(policy: Option<&Policy>) -> ApprovalContext<'_> {
    ApprovalContext {
        policy,
        signer_index: 2,
        now: 1_000,
    }
}

#[test]
fn keys_and_signatures_have_fips204_sizes() {
    for (level, pk_len, sig_len) in [
        (MlDsaLevel::MlDsa65, 1952, 3309),
        (MlDsaLevel::MlDsa87, 2592, 4627),
    ] {
        let ks = SoftwareKeystore::from_seed(level, [1; 32]);
        assert_eq!(ks.public_key().unwrap().bytes.len(), pk_len);
        assert_eq!(ks.sign(b"m").unwrap().bytes.len(), sig_len);
    }
}

#[test]
fn declined_request_is_never_signed() {
    let ks = SoftwareKeystore::from_seed(MlDsaLevel::MlDsa65, [1; 32]);
    let err = approve_withdraw(&ks, &Answer(false), &ctx(None), &message(2_000)).unwrap_err();
    assert!(matches!(err, SignerError::Declined));
}

#[test]
fn expired_request_is_refused_before_signing() {
    let ks = SoftwareKeystore::from_seed(MlDsaLevel::MlDsa65, [1; 32]);
    let err = approve_withdraw(&ks, &Answer(true), &ctx(None), &message(999)).unwrap_err();
    assert!(matches!(err, SignerError::PolicyViolation(_)));
}

#[test]
fn recipient_outside_known_allowlist_is_refused() {
    let ks = SoftwareKeystore::from_seed(MlDsaLevel::MlDsa65, [1; 32]);
    let policy = Policy::default(); // closed allowlist
    let err =
        approve_withdraw(&ks, &Answer(true), &ctx(Some(&policy)), &message(2_000)).unwrap_err();
    assert!(matches!(err, SignerError::PolicyViolation(_)));
}

#[test]
fn approval_carries_signer_index_and_matching_level() {
    let ks = SoftwareKeystore::from_seed(MlDsaLevel::MlDsa65, [1; 32]);
    let approval = approve_withdraw(&ks, &Answer(true), &ctx(None), &message(2_000)).unwrap();
    assert_eq!(approval.signer_index, 2);
    assert_eq!(approval.signature.level, MlDsaLevel::MlDsa65);
}

#[test]
#[ignore = "pre-alpha: requires an HSM with ML-DSA support (M4)"]
fn pkcs11_backend_signs_inside_token() {
    todo!()
}

#[cfg(unix)]
#[test]
fn keystore_with_open_permissions_is_refused() {
    use std::os::unix::fs::PermissionsExt;
    let dir = std::env::temp_dir().join(format!("qrypton-signer-test-{}", std::process::id()));
    std::fs::create_dir_all(&dir).unwrap();
    let path = dir.join("perm.qks");
    let _ = std::fs::remove_file(&path);

    SoftwareKeystore::generate(&path, MlDsaLevel::MlDsa65).unwrap();
    assert!(SoftwareKeystore::open(&path).is_ok());

    std::fs::set_permissions(&path, std::fs::Permissions::from_mode(0o644)).unwrap();
    assert!(matches!(
        SoftwareKeystore::open(&path),
        Err(SignerError::Backend(_))
    ));
    std::fs::remove_dir_all(&dir).unwrap();
}
