//! Canonical-encoding test vectors.
//!
//! The expected bytes below are written out from the format specification and
//! the XDR constants, not produced by the encoder under test. The contract
//! tests (`contracts/vault/src/test.rs`) additionally check that the on-chain
//! encoder produces the same bytes as this crate for the same request.

use qrypton_types::message::DomainTag;
use qrypton_types::{AddressBytes, MessageError, WithdrawMessage, WithdrawRequest};

/// XDR `ScVal::Address(ScAddress::Contract(id))`: SCV_ADDRESS = 18,
/// SC_ADDRESS_TYPE_CONTRACT = 1.
fn contract(id: u8) -> AddressBytes {
    let mut v = vec![0, 0, 0, 18, 0, 0, 0, 1];
    v.extend_from_slice(&[id; 32]);
    AddressBytes(v)
}

/// XDR `ScVal::Address(ScAddress::Account(Ed25519(key)))`: SCV_ADDRESS = 18,
/// SC_ADDRESS_TYPE_ACCOUNT = 0, PUBLIC_KEY_TYPE_ED25519 = 0.
fn account(key: u8) -> AddressBytes {
    let mut v = vec![0, 0, 0, 18, 0, 0, 0, 0, 0, 0, 0, 0];
    v.extend_from_slice(&[key; 32]);
    AddressBytes(v)
}

/// `SHA-256("Test SDF Network ; September 2015")`.
const TESTNET_ID: &str = "cee0302d59844d32bdca915c8203dd44b33fbb7edc19051ea37abedf28ecd472";

fn vector_message() -> WithdrawMessage {
    WithdrawMessage {
        network_id: hex::decode(TESTNET_ID).unwrap().try_into().unwrap(),
        vault: contract(0x11),
        request: WithdrawRequest {
            to: account(0x22),
            token: contract(0x33),
            amount: 1_000_000_000,
            nonce: 7,
            expires_at: 1_800_000_000,
        },
    }
}

fn vector_bytes() -> Vec<u8> {
    // u8 tag length (25) || "qrypton-vault/withdraw/v1"
    let mut v = vec![25u8];
    v.extend_from_slice(b"qrypton-vault/withdraw/v1");
    v.extend_from_slice(&hex::decode(TESTNET_ID).unwrap());
    // vault: u32 length 40 || contract address XDR
    v.extend_from_slice(&hex::decode("00000028").unwrap());
    v.extend_from_slice(&hex::decode("0000001200000001").unwrap());
    v.extend_from_slice(&[0x11; 32]);
    // to: u32 length 44 || account address XDR
    v.extend_from_slice(&hex::decode("0000002c").unwrap());
    v.extend_from_slice(&hex::decode("000000120000000000000000").unwrap());
    v.extend_from_slice(&[0x22; 32]);
    // token: u32 length 40 || contract address XDR
    v.extend_from_slice(&hex::decode("00000028").unwrap());
    v.extend_from_slice(&hex::decode("0000001200000001").unwrap());
    v.extend_from_slice(&[0x33; 32]);
    // amount: i128 big-endian (1_000_000_000)
    v.extend_from_slice(&hex::decode("0000000000000000000000003b9aca00").unwrap());
    // nonce: u64 big-endian (7)
    v.extend_from_slice(&hex::decode("0000000000000007").unwrap());
    // expires_at: u64 big-endian (1_800_000_000)
    v.extend_from_slice(&hex::decode("000000006b49d200").unwrap());
    v
}

#[test]
fn withdraw_v1_matches_specified_bytes() {
    let encoded = vector_message().to_bytes();
    assert_eq!(hex::encode(&encoded), hex::encode(vector_bytes()));
    assert_eq!(
        encoded.len(),
        1 + 25 + 32 + (4 + 40) + (4 + 44) + (4 + 40) + 16 + 8 + 8
    );
}

#[test]
fn withdraw_v1_roundtrip() {
    let msg = vector_message();
    assert_eq!(WithdrawMessage::from_bytes(&msg.to_bytes()), Ok(msg));
}

#[test]
fn withdraw_v1_rejects_trailing_bytes() {
    let mut bytes = vector_message().to_bytes();
    bytes.push(0);
    assert_eq!(
        WithdrawMessage::from_bytes(&bytes),
        Err(MessageError::TrailingBytes)
    );
}

#[test]
fn withdraw_v1_rejects_truncated_input() {
    let bytes = vector_message().to_bytes();
    assert_eq!(
        WithdrawMessage::from_bytes(&bytes[..bytes.len() - 1]),
        Err(MessageError::Truncated)
    );
}

#[test]
fn withdraw_v1_rejects_other_domain() {
    let mut bytes = vector_message().to_bytes();
    bytes[1..26].copy_from_slice(b"qrypton-vault/withdraw/v2");
    assert_eq!(
        WithdrawMessage::from_bytes(&bytes),
        Err(MessageError::WrongDomain)
    );
}

#[test]
fn domain_tags_are_distinct() {
    let tags = [
        DomainTag::WithdrawV1,
        DomainTag::SetOperatorV1,
        DomainTag::RotateKeysV1,
        DomainTag::SetPolicyV1,
        DomainTag::CancelOperatorChangeV1,
        DomainTag::CancelKeyRotationV1,
        DomainTag::CancelPolicyChangeV1,
    ];
    for (i, a) in tags.iter().enumerate() {
        for b in &tags[i + 1..] {
            assert_ne!(a.as_bytes(), b.as_bytes());
        }
    }
}

#[cfg(feature = "stellar")]
mod stellar {
    use super::*;
    use qrypton_types::stellar::network_id;

    #[test]
    fn network_id_matches_testnet() {
        assert_eq!(
            hex::encode(network_id("Test SDF Network ; September 2015")),
            TESTNET_ID
        );
    }

    #[test]
    fn strkey_encodes_as_scval_address_xdr() {
        // All-zero Ed25519 key and contract id.
        let g =
            AddressBytes::from_strkey("GAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAWHF")
                .unwrap();
        assert_eq!(g, account(0));
        let c =
            AddressBytes::from_strkey("CAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAABSC4")
                .unwrap();
        assert_eq!(c, contract(0));
        assert_eq!(
            g.to_strkey().unwrap(),
            "GAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAWHF"
        );
    }
}
