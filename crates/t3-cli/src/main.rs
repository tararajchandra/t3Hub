mod commands;
mod crypto;
mod manifest;

use clap::{Parser, Subcommand};
use commands::digest::execute_digest;
use commands::index_generate::{execute_index_generate, IndexGenOptions};
use commands::init::{execute_init, InitOptions};
use commands::keygen::execute_keygen;
use commands::sign::execute_sign;
use commands::validate::execute_validate;
use commands::verify::execute_verify;
use std::process;

#[derive(Parser)]
#[command(
    name = "t3",
    author = "T3 Hub Team",
    version = "0.1.0",
    about = "Developer CLI for the T3 Hub package ecosystem (T3P v1.0)",
    long_about = "T3 Hub CLI enables developers to author, validate, hash, sign, and publish software packages for the T3 Hub ecosystem."
)]
struct Cli {
    #[command(subcommand)]
    command: Commands,
}

#[derive(Subcommand)]
enum Commands {
    /// Initialize a new T3 Hub package manifest (manifest.yaml)
    Init {
        /// Reverse-DNS package identifier (e.g. 'developer.devstudio')
        #[arg(long)]
        id: Option<String>,

        /// Display name of the application
        #[arg(long)]
        name: Option<String>,

        /// Semantic version (e.g. '1.0.0')
        #[arg(long)]
        version: Option<String>,

        /// Publisher or company name
        #[arg(long)]
        publisher: Option<String>,

        /// Package installer type: exe, msi, or zip
        #[arg(long, rename_all = "lower")]
        r#type: Option<String>,

        /// Installation scope: user or machine
        #[arg(long, rename_all = "lower")]
        scope: Option<String>,

        /// Download URL for the artifact
        #[arg(long)]
        url: Option<String>,

        /// 64-character SHA-256 hash of the artifact
        #[arg(long)]
        sha256: Option<String>,

        /// Output file path
        #[arg(short, long, default_value = "manifest.yaml")]
        output: String,
    },

    /// Validate one or more package manifests against the T3P v1.0 specification
    Validate {
        /// Path to a manifest YAML file or a directory containing manifests
        path: String,
    },

    /// Calculate streaming SHA-256 digest and byte size of an installer artifact
    Digest {
        /// Path to the artifact binary (.exe, .msi, or .zip)
        file: String,
    },

    /// Generate a new Ed25519 signing keypair for repository signing
    Keygen {
        /// Optional JSON output file path to save the generated keypair
        #[arg(short, long)]
        output: Option<String>,
    },

    /// Sign a file (index.json or manifest) using an Ed25519 secret key
    Sign {
        /// Path to the file to sign
        file: String,

        /// 32-byte secret signing key (64 hex characters)
        #[arg(short, long)]
        key: String,

        /// Optional output file for the detached signature
        #[arg(short, long)]
        output: Option<String>,
    },

    /// Verify an Ed25519 detached signature against a file
    Verify {
        /// Path to the file to verify
        file: String,

        /// Detached signature hex string or path to a signature file
        #[arg(short, long)]
        sig: String,

        /// 32-byte Ed25519 public key (64 hex characters)
        #[arg(short, long)]
        public_key: String,
    },

    /// Scan repository manifests and generate or update index.json
    IndexGenerate {
        /// Path to repository root directory (must contain 'manifests/' folder)
        #[arg(short, long, default_value = ".")]
        repo_dir: String,

        /// Repository ID override (defaults to repository.json 'id' or 'community-repo')
        #[arg(long)]
        repo_id: Option<String>,

        /// Custom output path for index.json
        #[arg(short, long)]
        output: Option<String>,

        /// Optional secret key hex to automatically generate a signed index (index.json.sig)
        #[arg(short, long)]
        sign_key: Option<String>,
    },
}

fn main() {
    let cli = Cli::parse();

    let result = match cli.command {
        Commands::Init {
            id,
            name,
            version,
            publisher,
            r#type,
            scope,
            url,
            sha256,
            output,
        } => execute_init(InitOptions {
            id,
            name,
            version,
            publisher,
            install_type: r#type,
            scope,
            url,
            sha256,
            output,
        }),

        Commands::Validate { path } => execute_validate(&path),

        Commands::Digest { file } => execute_digest(&file),

        Commands::Keygen { output } => execute_keygen(output.as_deref()),

        Commands::Sign { file, key, output } => {
            execute_sign(&file, &key, output.as_deref())
        }

        Commands::Verify {
            file,
            sig,
            public_key,
        } => execute_verify(&file, &sig, &public_key),

        Commands::IndexGenerate {
            repo_dir,
            repo_id,
            output,
            sign_key,
        } => execute_index_generate(IndexGenOptions {
            repo_dir,
            repo_id,
            output,
            sign_key,
        }),
    };

    if let Err(e) = result {
        eprintln!("\nError: {}", e);
        process::exit(1);
    }
}
