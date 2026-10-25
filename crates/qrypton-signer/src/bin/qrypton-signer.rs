//! `qrypton-signer` — command-line approver for Qrypton Vault.
//!
//! ```text
//! qrypton-signer keygen          --keystore approver.qks [--algorithm ml-dsa-65|ml-dsa-87]
//! qrypton-signer show-pubkey     --keystore approver.qks
//! qrypton-signer approve-withdraw --keystore approver.qks --request request.json \
//!                                 --signer-index 0 --rpc-url <url> --out approval.json
//! ```
//!
//! `--pkcs11-module`, `--token-label` and `--key-label` select an HSM instead
//! of a software keystore.

use std::path::PathBuf;

use clap::{Args, Parser, Subcommand, ValueEnum};

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
    },
    /// Review a withdrawal request, pre-check it against the vault policy and,
    /// on confirmation, produce a signed approval.
    ApproveWithdraw {
        #[command(flatten)]
        key: KeySource,
        /// Withdrawal request file produced by the SDK.
        #[arg(long)]
        request: PathBuf,
        /// This signer's index in the vault's signer set.
        #[arg(long)]
        signer_index: u32,
        /// Stellar RPC endpoint used to read the vault policy.
        #[arg(long)]
        rpc_url: String,
        /// Where to write the approval.
        #[arg(long)]
        out: PathBuf,
    },
}

/// Where the key lives: a software keystore or a PKCS#11 HSM.
#[derive(Args)]
struct KeySource {
    /// Software keystore file.
    #[arg(long, conflicts_with = "pkcs11_module")]
    keystore: Option<PathBuf>,
    /// PKCS#11 module (shared library) of the HSM.
    #[arg(long, requires_all = ["token_label", "key_label"])]
    pkcs11_module: Option<PathBuf>,
    /// HSM token label.
    #[arg(long)]
    token_label: Option<String>,
    /// Label of the ML-DSA key on the token.
    #[arg(long)]
    key_label: Option<String>,
}

#[derive(Clone, Copy, ValueEnum)]
enum Algorithm {
    #[value(name = "ml-dsa-65")]
    MlDsa65,
    #[value(name = "ml-dsa-87")]
    MlDsa87,
}

fn main() {
    let cli = Cli::parse();
    match cli.command {
        Command::Keygen { .. } => todo!("SoftwareKeystore::generate"),
        Command::ShowPubkey { .. } => todo!("open backend, print public key"),
        Command::ApproveWithdraw { .. } => todo!("approval::approve_withdraw"),
    }
}
