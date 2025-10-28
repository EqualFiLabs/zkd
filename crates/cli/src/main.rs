use anyhow::{anyhow, Context, Result};
use clap::{Args, Parser, Subcommand};
use std::convert::TryFrom;
use std::fs;
use std::path::Path;
use std::process;
use zkprov_backend_native::{native_prove, native_verify};
use zkprov_corelib as core;
use zkprov_corelib::air::AirProgram;
use zkprov_corelib::air_bindings::Bindings;
use zkprov_corelib::config::Config;
use zkprov_corelib::evm::digest::digest_D;
use zkprov_corelib::gadgets::commitment::{
    Comm32, CommitmentScheme32, PedersenParams, PedersenPlaceholder, Witness,
};
use zkprov_corelib::proof::ProofHeader;
use zkprov_corelib::registry;
use zkprov_corelib::trace::TraceShape;
use zkprov_corelib::validate::{validate_air_against_backend, validate_config};
use zkprov_corelib::validation::Validator;

const EXIT_CORRUPT_PROOF: i32 = 4;

#[derive(Parser)]
#[command(name = "zkd", version, about = "ZKProv CLI")]
struct Cli {
    #[command(subcommand)]
    command: Option<Commands>,
}

#[derive(Args, Debug, Clone)]
struct CommonCfg {
    /// Backend id, e.g. native@0.0
    #[arg(long = "backend")]
    backend_id: String,
    /// Field id, e.g. Prime254
    #[arg(long = "field")]
    field: String,
    /// Hash id, e.g. blake3
    #[arg(long = "hash")]
    hash: String,
    /// FRI arity (2,4,...)
    #[arg(long = "fri-arity")]
    fri_arity: u32,
    /// Require recursion capability (fails if backend doesn't support)
    #[arg(long = "need-recursion", default_value_t = false)]
    need_recursion: bool,
    /// Profile id, e.g. balanced
    #[arg(long = "profile")]
    profile_id: String,
}

#[derive(Subcommand)]
enum Commands {
    /// List available backends
    BackendLs {
        /// Show full capability matrix
        #[arg(short, long)]
        verbose: bool,
    },
    /// List available profiles
    ProfileLs,
    /// Print the public I/O schema derived from the program AIR
    IoSchema {
        /// Program AIR path (.air TOML)
        #[arg(short = 'p', long = "program")]
        program_path: String,
        /// Emit JSON (default) or pretty JSON
        #[arg(long = "pretty", default_value_t = false)]
        pretty: bool,
    },
    /// Prove: read inputs JSON, produce proof blob
    Prove {
        /// Program AIR path (.air TOML)
        #[arg(short = 'p', long = "program")]
        program_path: String,
        /// Inputs JSON path
        #[arg(short = 'i', long = "inputs")]
        inputs_path: String,
        /// Output proof file path
        #[arg(short = 'o', long = "output")]
        proof_out: String,
        /// Print stats row/col/body_len after success
        #[arg(long = "stats", default_value_t = false)]
        stats: bool,
        #[command(flatten)]
        cfg: CommonCfg,
    },
    /// Verify: read inputs JSON and proof blob, return success/failure
    Verify {
        /// Program AIR path (.air TOML)
        #[arg(short = 'p', long = "program")]
        program_path: String,
        /// Inputs JSON path
        #[arg(short = 'i', long = "inputs")]
        inputs_path: String,
        /// Proof file path
        #[arg(short = 'P', long = "proof")]
        proof_in: String,
        /// Print stats row/col/body_len after success
        #[arg(long = "stats", default_value_t = false)]
        stats: bool,
        #[command(flatten)]
        cfg: CommonCfg,
    },
    /// Validate: derive commitment checks and emit a structured report
    Validate {
        /// Program AIR path (.air TOML)
        #[arg(short = 'p', long = "program")]
        program_path: String,
        /// Inputs JSON path
        #[arg(short = 'i', long = "inputs")]
        inputs_path: String,
        /// Proof file path
        #[arg(short = 'P', long = "proof")]
        proof_in: String,
        /// Output directory for validation reports
        #[arg(short = 'o', long = "output")]
        output_dir: String,
        #[command(flatten)]
        cfg: CommonCfg,
    },
    /// Compute a Pedersen (placeholder) commitment for msg/blind (hex).
    Commit {
        #[arg(long = "hash")]
        hash_id: String,
        #[arg(long = "msg-hex")]
        msg_hex: String,
        #[arg(long = "blind-hex")]
        blind_hex: String,
    },
    /// Verify opening against a commitment (all hex).
    OpenCommit {
        #[arg(long = "hash")]
        hash_id: String,
        #[arg(long = "msg-hex")]
        msg_hex: String,
        #[arg(long = "blind-hex")]
        blind_hex: String,
        #[arg(long = "commit-hex")]
        commit_hex: String,
    },
    /// Compute the Keccak digest (D) used by the EVM verifier from a proof blob.
    EvmDigest {
        /// Proof file path
        #[arg(short = 'P', long = "proof")]
        proof_path: String,
    },
}

fn read_to_string(path: &str) -> Result<String> {
    let content = fs::read_to_string(path).with_context(|| format!("failed to read '{}'", path))?;
    Ok(content)
}

fn read_to_bytes(path: &str) -> Result<Vec<u8>> {
    let bytes = fs::read(path).with_context(|| format!("failed to read '{}'", path))?;
    Ok(bytes)
}

fn write_bytes(path: &str, bytes: &[u8]) -> Result<()> {
    if let Some(dir) = Path::new(path).parent() {
        if !dir.as_os_str().is_empty() {
            fs::create_dir_all(dir)
                .with_context(|| format!("failed to create dir '{}'", dir.display()))?;
        }
    }
    fs::write(path, bytes).with_context(|| format!("failed to write '{}'", path))?;
    Ok(())
}

fn mk_config(c: &CommonCfg) -> Config {
    Config::new(
        &c.backend_id,
        &c.field,
        &c.hash,
        c.fri_arity,
        c.need_recursion,
        &c.profile_id,
    )
}

/// Map verifier/proof parsing failures to the mandated exit code (4).
fn exit_for_corrupt_proof(err: &anyhow::Error) -> ! {
    eprintln!("Error: {err}");
    process::exit(EXIT_CORRUPT_PROOF);
}

// --- Hex helpers ---------------------------------------------------------

fn hex_to_bytes(s: &str) -> Result<Vec<u8>> {
    if !s.len().is_multiple_of(2) {
        return Err(anyhow!("hex string has odd length"));
    }
    let mut out = Vec::with_capacity(s.len() / 2);
    let bytes = s.as_bytes();
    for i in (0..bytes.len()).step_by(2) {
        let hi = hex_val(bytes[i])?;
        let lo = hex_val(bytes[i + 1])?;
        out.push((hi << 4) | lo);
    }
    Ok(out)
}

fn hex_val(b: u8) -> Result<u8> {
    match b {
        b'0'..=b'9' => Ok(b - b'0'),
        b'a'..=b'f' => Ok(b - b'a' + 10),
        b'A'..=b'F' => Ok(b - b'A' + 10),
        _ => Err(anyhow!("invalid hex char")),
    }
}

fn bytes_to_hex(v: &[u8]) -> String {
    const HEX: &[u8; 16] = b"0123456789abcdef";
    let mut out = String::with_capacity(v.len() * 2);
    for &b in v {
        out.push(HEX[(b >> 4) as usize] as char);
        out.push(HEX[(b & 0x0f) as usize] as char);
    }
    out
}

fn main() -> Result<()> {
    let cli = Cli::parse();
    match cli.command {
        Some(Commands::BackendLs { verbose }) => {
            let infos = core::list_backends();
            if !verbose {
                for b in infos {
                    println!("{}  recursion={}", b.id, b.recursion);
                }
            } else {
                for b in infos {
                    let caps =
                        registry::get_backend_capabilities(b.id).expect("backend disappeared");
                    println!("{}", b.id);
                    println!("  recursion: {}", caps.recursion);
                    println!("  lookups: {}", caps.lookups);
                    println!("  fields: {}", caps.fields.join(", "));
                    println!("  hashes: {}", caps.hashes.join(", "));
                    let arities = caps
                        .fri_arities
                        .iter()
                        .map(|a| a.to_string())
                        .collect::<Vec<_>>()
                        .join(", ");
                    println!("  fri_arities: {}", arities);
                }
            }
        }
        Some(Commands::ProfileLs) => {
            let profiles = core::list_profiles();
            for p in profiles {
                println!("{}  λ={} bits", p.id, p.lambda_bits);
            }
        }
        Some(Commands::IoSchema {
            program_path,
            pretty,
        }) => {
            let air = AirProgram::load_from_file(&program_path)?;
            let shape = TraceShape::from_air(&air);
            // Minimal schema reflection for Phase-0 (public inputs remain free-form JSON)
            let schema = serde_json::json!({
                "program": air.meta.name,
                "field": air.meta.field,
                "hash": format!("{:?}", air.meta.hash).to_lowercase(),
                "trace": { "rows": shape.rows, "cols": shape.cols, "const_cols": shape.const_cols, "periodic_cols": shape.periodic_cols },
                "public_inputs": { "kind": "json", "binding": "raw" },
                "commitments": {
                    "pedersen": true,
                    "curves": ["placeholder"],
                    "no_r_reuse": false
                }
            });
            if pretty {
                println!("{}", serde_json::to_string_pretty(&schema)?);
            } else {
                println!("{}", serde_json::to_string(&schema)?);
            }
        }
        Some(Commands::Prove {
            program_path,
            inputs_path,
            proof_out,
            stats,
            cfg,
        }) => {
            registry::ensure_builtins_registered();
            let config = mk_config(&cfg);
            validate_config(&config).map_err(|e| anyhow!(e.to_string()))?;
            let inputs = read_to_string(&inputs_path)?;

            if config.backend_id == "native@0.0" {
                let proof = native_prove(&config, &inputs, &program_path)?;
                write_bytes(&proof_out, &proof)?;
                let hdr = ProofHeader::decode(&proof[0..40])
                    .unwrap_or_else(|e| exit_for_corrupt_proof(&e));
                println!(
                    "✅ ProofGenerated backend={} profile={} body_len={} pubio_hash=0x{:016x}",
                    config.backend_id, config.profile_id, hdr.body_len, hdr.pubio_hash
                );
                if stats {
                    let air = AirProgram::load_from_file(&program_path)?;
                    let shape = TraceShape::from_air(&air);
                    println!(
                        "stats rows={} cols={} const={} periodic={}",
                        shape.rows, shape.cols, shape.const_cols, shape.periodic_cols
                    );
                }
                println!("Program: {}", program_path);
                println!("Wrote: {}", proof_out);
            } else {
                return Err(anyhow!(
                    "backend '{}' not implemented yet in CLI",
                    config.backend_id
                ));
            }
        }
        Some(Commands::Verify {
            program_path,
            inputs_path,
            proof_in,
            stats,
            cfg,
        }) => {
            registry::ensure_builtins_registered();
            let config = mk_config(&cfg);
            validate_config(&config).map_err(|e| anyhow!(e.to_string()))?;
            let inputs = read_to_string(&inputs_path)?;
            let proof = read_to_bytes(&proof_in)?;

            if config.backend_id == "native@0.0" {
                // First, attempt to decode header; any failure maps to exit code 4
                let hdr = match ProofHeader::decode(proof.get(0..40).unwrap_or(&[])) {
                    Ok(h) => h,
                    Err(e) => exit_for_corrupt_proof(&e),
                };
                // Now run backend verify; any transcript/commit mismatch is also "corrupt proof"
                match native_verify(&config, &inputs, &program_path, &proof) {
                    Ok(true) => {
                        println!(
                            "✅ ProofVerified backend={} profile={} pubio_hash=0x{:016x}",
                            config.backend_id, config.profile_id, hdr.pubio_hash
                        );
                        if stats {
                            let air = AirProgram::load_from_file(&program_path)?;
                            let shape = TraceShape::from_air(&air);
                            println!(
                                "stats rows={} cols={} const={} periodic={}",
                                shape.rows, shape.cols, shape.const_cols, shape.periodic_cols
                            );
                        }
                    }
                    Ok(false) => {
                        eprintln!("❌ Verification failed");
                        process::exit(EXIT_CORRUPT_PROOF);
                    }
                    Err(e) => {
                        // Treat mismatches and root/header problems as "corrupt proof"
                        exit_for_corrupt_proof(&e);
                    }
                }
            } else {
                return Err(anyhow!(
                    "backend '{}' not implemented yet in CLI",
                    config.backend_id
                ));
            }
        }
        Some(Commands::Validate {
            program_path,
            inputs_path,
            proof_in,
            output_dir,
            cfg,
        }) => {
            registry::ensure_builtins_registered();
            let config = mk_config(&cfg);
            validate_config(&config).map_err(|e| anyhow!(e.to_string()))?;
            let air = AirProgram::load_from_file(&program_path)?;
            validate_air_against_backend(&air, &config.backend_id)
                .map_err(|e| anyhow!(e.to_string()))?;
            let bindings = Bindings::from_air(&air);

            let proof = read_to_bytes(&proof_in)?;
            if proof.len() < 40 {
                return Err(anyhow!(
                    "proof '{}' is too short for header ({} bytes)",
                    proof_in,
                    proof.len()
                ));
            }
            let header = ProofHeader::decode(&proof[0..40])
                .map_err(|e| anyhow!("failed to decode proof header: {e}"))?;
            let body = &proof[40..];
            if body.len() as u64 != header.body_len {
                return Err(anyhow!(
                    "proof '{}' body length ({}) does not match header body_len {}",
                    proof_in,
                    body.len(),
                    header.body_len
                ));
            }

            let inputs_json = read_to_string(&inputs_path)?;
            let mut validator = Validator::new(&bindings);

            if bindings.commitments.pedersen {
                let mut msg_bytes = inputs_json.into_bytes();
                msg_bytes.extend_from_slice(body);
                let mut blind_bytes = Vec::new();
                blind_bytes.extend_from_slice(&header.pubio_hash.to_le_bytes());
                blind_bytes.extend_from_slice(&header.backend_id_hash.to_le_bytes());
                blind_bytes.extend_from_slice(&header.profile_id_hash.to_le_bytes());
                validator.check_commit_point(&msg_bytes, &blind_bytes);
            }
            validator.check_range_u64(header.body_len, 64);

            let mut report = validator.finalize();
            report.meta.backend_id = config.backend_id.clone();
            report.meta.profile_id = config.profile_id.clone();
            report.meta.hash_id = bindings
                .hash_id_for_commitments
                .clone()
                .unwrap_or_else(|| config.hash.clone());
            report.meta.curve = bindings.commitments.curve.clone();

            let report_path = report.write_pretty(&output_dir).with_context(|| {
                format!("failed to write validation report under '{}'", output_dir)
            })?;
            println!(
                "✅ Validation ok={} commit_passed={} report={}",
                report.ok,
                report.commit_passed,
                report_path.display()
            );
            if !report.ok {
                for err in &report.errors {
                    eprintln!("❌ {:?}: {}", err.code, err.msg);
                }
                process::exit(1);
            }
        }
        Some(Commands::Commit {
            hash_id,
            msg_hex,
            blind_hex,
        }) => {
            registry::ensure_builtins_registered();
            let msg = hex_to_bytes(&msg_hex)?;
            let blind = hex_to_bytes(&blind_hex)?;
            let ped = PedersenPlaceholder::new(PedersenParams { hash_id });
            let commitment = ped.commit(&Witness {
                msg: &msg,
                blind: &blind,
            })?;
            println!("{}", bytes_to_hex(commitment.as_bytes()));
        }
        Some(Commands::OpenCommit {
            hash_id,
            msg_hex,
            blind_hex,
            commit_hex,
        }) => {
            registry::ensure_builtins_registered();
            let msg = hex_to_bytes(&msg_hex)?;
            let blind = hex_to_bytes(&blind_hex)?;
            let cbytes = hex_to_bytes(&commit_hex)?;
            if cbytes.len() != 32 {
                return Err(anyhow!("commit-hex must be 32 bytes (64 hex chars)"));
            }
            let mut c32 = [0u8; 32];
            c32.copy_from_slice(&cbytes);
            let ped = PedersenPlaceholder::new(PedersenParams { hash_id });
            let opened = ped.open(
                &Witness {
                    msg: &msg,
                    blind: &blind,
                },
                &Comm32(c32),
            )?;
            if opened {
                println!("✅ Opened");
            } else {
                println!("❌ Invalid opening");
                process::exit(1);
            }
        }
        Some(Commands::EvmDigest { proof_path }) => {
            let proof = read_to_bytes(&proof_path)?;
            if proof.len() < 40 {
                return Err(anyhow!(
                    "proof '{}' is too short for header ({} bytes)",
                    proof_path,
                    proof.len()
                ));
            }
            let header = ProofHeader::decode(&proof[0..40])?;
            let body_len = usize::try_from(header.body_len).map_err(|_| {
                anyhow!("header body_len {} does not fit in memory", header.body_len)
            })?;
            let expected_len = 40usize
                .checked_add(body_len)
                .ok_or_else(|| anyhow!("proof length overflow"))?;
            if proof.len() != expected_len {
                return Err(anyhow!(
                    "proof '{}' length ({}) does not match header body_len {}",
                    proof_path,
                    proof.len(),
                    header.body_len
                ));
            }
            let body = &proof[40..expected_len];
            let digest = digest_D(&header, body);
            println!("0x{}", bytes_to_hex(&digest));
        }
        None => {
            println!("zkd {} — ready", core::version());
            println!("Try: `zkd backend-ls [-v]`, `zkd profile-ls`,");
            println!("     `zkd io-schema -p <program.air>`,",);
            println!("     `zkd commit --hash <id> --msg-hex <..> --blind-hex <..>`,",);
            println!(
                "     `zkd open-commit --hash <id> --msg-hex <..> --blind-hex <..> --commit-hex <..>`,",
            );
            println!(
                "     `zkd prove -p <program> -i <inputs> -o <proof> --profile ... [--stats]`,",
            );
            println!(
                "     `zkd verify -p <program> -i <inputs> -P <proof> --profile ... [--stats]`",
            );
            println!(
                "     `zkd validate -p <program> -i <inputs> -P <proof> -o <reports> --profile ...`",
            );
        }
    }
    Ok(())
}
