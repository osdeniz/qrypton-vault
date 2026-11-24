//! `qrypton-signer` — command-line approver for Qrypton Vault.
//!
//! ```text
//! qrypton-signer keygen           --keystore approver.qks [--algorithm ml-dsa-65|ml-dsa-87]
//! qrypton-signer show-pubkey      --keystore approver.qks [--json]
//! qrypton-signer approve-withdraw --keystore approver.qks --signer-index 0 \
//!     --network-passphrase "Test SDF Network ; September 2015" \
//!     --vault C... --to G... --token C... --amount 10000000 --nonce 1 \
//!     --expires-at 1800000000 --out approval.json [--yes]
//! ```
//!
//! The approval file holds one `PqApproval` in the JSON form `stellar contract
//! invoke` accepts, e.g. `{"signer_index":0,"signature":{"MlDsa65":"<hex>"}}`.
//!
//! The software keystore is **unencrypted** and for development only;
//! encryption at rest arrives in M2. The PKCS#11 (HSM) backend is not
//! implemented yet.

use std::io::{BufRead, Write};
use std::path::PathBuf;
use std::process::ExitCode;
use std::time::{SystemTime, UNIX_EPOCH};

use clap::{Args, Parser, Subcommand, ValueEnum};
use qrypton_signer::approval::{approve_withdraw, ApprovalContext, Confirm};
use qrypton_signer::backend::{KeyBackend, SoftwareKeystore};
use qrypton_signer::SignerError;
use qrypton_types::stellar::network_id;
use qrypton_types::{AddressBytes, MlDsaLevel, PqApproval, WithdrawMessage, WithdrawRequest};
use sha2::{Digest, Sha256};

#[derive(Parser)]
#[command(
    name = "qrypton-signer",
    version,
    about = "Qrypton Vault post-quantum approver"
)]
struct Cli {
    #[command(subcommand)]
    command: Command,
}

#[derive(Subcommand)]
enum Command {
    /// Generate a new ML-DSA key in a software keystore (development only).
    Keygen {
        /// Keystore file to create.
        #[arg(long)]
        keystore: PathBuf,
        /// ML-DSA parameter set.
        #[arg(long, value_enum, default_value_t = Algorithm::MlDsa65)]
        algorithm: Algorithm,
    },
    /// Print the encoded public key to register in the vault's signer set.
    ShowPubkey {
        #[command(flatten)]
        key: KeySource,
        /// Print the `PqPublicKey` JSON accepted by `stellar contract` instead of hex.
        #[arg(long)]
        json: bool,
    },
    /// Review a withdrawal request, pre-check it and, on confirmation, write a
    /// signed approval.
    ApproveWithdraw {
        #[command(flatten)]
        key: KeySource,
        /// This signer's index in the vault's signer set.
        #[arg(long)]
        signer_index: u32,
        #[command(flatten)]
        request: RequestArgs,
        /// Where to write the approval.
        #[arg(long)]
        out: PathBuf,
        /// Approve without the interactive confirmation prompt.
        #[arg(long)]
        yes: bool,
    },
}

/// Where the key lives: a software keystore or (not yet implemented) a
/// PKCS#11 HSM.
#[derive(Args)]
struct KeySource {
    /// Software keystore file.
    #[arg(long, conflicts_with = "pkcs11_module")]
    keystore: Option<PathBuf>,
    /// PKCS#11 module (shared library) of the HSM. Not implemented yet.
    #[arg(long, requires_all = ["token_label", "key_label"])]
    pkcs11_module: Option<PathBuf>,
    /// HSM token label.
    #[arg(long)]
    token_label: Option<String>,
    /// Label of the ML-DSA key on the token.
    #[arg(long)]
    key_label: Option<String>,
}

#[derive(Args)]
struct RequestArgs {
    /// Network passphrase, e.g. "Test SDF Network ; September 2015".
    #[arg(long)]
    network_passphrase: String,
    /// Vault contract address (C...).
    #[arg(long)]
    vault: String,
    /// Recipient (G... or C...).
    #[arg(long)]
    to: String,
    /// Token contract (C...).
    #[arg(long)]
    token: String,
    /// Amount in the token's smallest unit.
    #[arg(long)]
    amount: i128,
    /// Single-use nonce.
    #[arg(long)]
    nonce: u64,
    /// Ledger timestamp (UNIX seconds) after which the request is invalid.
    #[arg(long)]
    expires_at: u64,
}

#[derive(Clone, Copy, ValueEnum)]
enum Algorithm {
    #[value(name = "ml-dsa-65")]
    MlDsa65,
    #[value(name = "ml-dsa-87")]
    MlDsa87,
}

impl From<Algorithm> for MlDsaLevel {
    fn from(a: Algorithm) -> Self {
        match a {
            Algorithm::MlDsa65 => MlDsaLevel::MlDsa65,
            Algorithm::MlDsa87 => MlDsaLevel::MlDsa87,
        }
    }
}

fn main() -> ExitCode {
    match run(Cli::parse()) {
        Ok(()) => ExitCode::SUCCESS,
        Err(e) => {
            eprintln!("error: {e}");
            ExitCode::FAILURE
        }
    }
}

fn run(cli: Cli) -> Result<(), SignerError> {
    match cli.command {
        Command::Keygen {
            keystore,
            algorithm,
        } => {
            let ks = SoftwareKeystore::generate(&keystore, algorithm.into())?;
            eprintln!(
                "WARNING: {} is an UNENCRYPTED development keystore (mode 0600). Do not use it for real funds.",
                keystore.display()
            );
            println!("{}", hex::encode(ks.public_key()?.bytes));
        }
        Command::ShowPubkey { key, json } => {
            let pk = open_backend(&key)?.public_key()?;
            let hex = hex::encode(&pk.bytes);
            if json {
                println!("{{\"{}\":\"{hex}\"}}", variant(pk.level));
            } else {
                println!("{hex}");
            }
        }
        Command::ApproveWithdraw {
            key,
            signer_index,
            request,
            out,
            yes,
        } => {
            let backend = open_backend(&key)?;
            let message = build_message(&request)?;
            let now = SystemTime::now()
                .duration_since(UNIX_EPOCH)
                .map_err(|_| SignerError::Backend("system clock is before 1970".into()))?
                .as_secs();
            let ctx = ApprovalContext {
                policy: None,
                signer_index,
                now,
            };
            let approval = approve_withdraw(&*backend, &TerminalConfirm { yes }, &ctx, &message)?;
            std::fs::write(&out, approval_json(&approval))?;
            eprintln!("approval written to {}", out.display());
        }
    }
    Ok(())
}

fn open_backend(key: &KeySource) -> Result<Box<dyn KeyBackend>, SignerError> {
    match (&key.keystore, &key.pkcs11_module) {
        (Some(path), None) => Ok(Box::new(SoftwareKeystore::open(path)?)),
        (None, Some(_)) => Err(SignerError::Backend(
            "the PKCS#11 (HSM) backend is not implemented yet".into(),
        )),
        _ => Err(SignerError::Backend(
            "specify --keystore (or --pkcs11-module)".into(),
        )),
    }
}

fn build_message(r: &RequestArgs) -> Result<WithdrawMessage, SignerError> {
    Ok(WithdrawMessage {
        network_id: network_id(&r.network_passphrase),
        vault: AddressBytes::from_strkey(&r.vault)?,
        request: WithdrawRequest {
            to: AddressBytes::from_strkey(&r.to)?,
            token: AddressBytes::from_strkey(&r.token)?,
            amount: r.amount,
            nonce: r.nonce,
            expires_at: r.expires_at,
        },
    })
}

fn variant(level: MlDsaLevel) -> &'static str {
    match level {
        MlDsaLevel::MlDsa44 => "MlDsa44",
        MlDsaLevel::MlDsa65 => "MlDsa65",
        MlDsaLevel::MlDsa87 => "MlDsa87",
    }
}

fn approval_json(a: &PqApproval) -> String {
    format!(
        "{{\"signer_index\":{},\"signature\":{{\"{}\":\"{}\"}}}}\n",
        a.signer_index,
        variant(a.signature.level),
        hex::encode(&a.signature.bytes)
    )
}

/// Shows the decoded request on stderr and asks for confirmation.
struct TerminalConfirm {
    yes: bool,
}

impl Confirm for TerminalConfirm {
    fn confirm(&self, m: &WithdrawMessage) -> Result<bool, SignerError> {
        let bytes = m.to_bytes();
        let r = &m.request;
        eprintln!("Withdrawal request (qrypton-vault/withdraw/v1)");
        eprintln!("  network id : {}", hex::encode(m.network_id));
        eprintln!("  vault      : {}", m.vault.to_strkey()?);
        eprintln!("  to         : {}", r.to.to_strkey()?);
        eprintln!("  token      : {}", r.token.to_strkey()?);
        eprintln!("  amount     : {}", r.amount);
        eprintln!("  nonce      : {}", r.nonce);
        eprintln!("  expires at : {}", r.expires_at);
        eprintln!(
            "  message    : {} bytes, sha256 {}",
            bytes.len(),
            hex::encode(Sha256::digest(&bytes))
        );
        eprintln!("  note       : on-chain policy pre-check needs the SDK (M3); only expiry is checked here");
        if self.yes {
            return Ok(true);
        }
        eprint!("Approve this withdrawal? [y/N] ");
        std::io::stderr().flush()?;
        let mut line = String::new();
        std::io::stdin().lock().read_line(&mut line)?;
        Ok(matches!(line.trim(), "y" | "Y" | "yes"))
    }
}
