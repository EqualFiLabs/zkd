use anyhow::{anyhow, Context, Result};
use clap::{Args, Parser, Subcommand};
use std::fs;
use std::path::Path;
use std::process;
use zkprov_backend_native::{native_prove, native_verify};
use zkprov_corelib as core;
use zkprov_corelib::air::AirProgram;
use zkprov_corelib::config::Config;
use zkprov_corelib::proof::ProofHeader;
use zkprov_corelib::registry;
use zkprov_corelib::trace::TraceShape;
use zkprov_corelib::validate::validate_config;

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
                "public_inputs": { "kind": "json", "binding": "raw" }
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
                    .unwrap_or_else(|e| exit_for_corrupt_proof(&e.into()));
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
                    Err(e) => exit_for_corrupt_proof(&e.into()),
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
        None => {
            println!("zkd {} — ready", core::version());
            println!("Try: `zkd backend-ls [-v]`, `zkd profile-ls`,");
            println!("     `zkd io-schema -p <program.air>`,",);
            println!(
                "     `zkd prove -p <program> -i <inputs> -o <proof> --profile ... [--stats]`,",
            );
            println!(
                "     `zkd verify -p <program> -i <inputs> -P <proof> --profile ... [--stats]`",
            );
        }
    }
    Ok(())
}
