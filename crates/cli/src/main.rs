use anyhow::Result;
use clap::{Parser, Subcommand};
use zkprov_corelib as core;

#[derive(Parser)]
#[command(name = "zkd", version, about = "ZKProv CLI")]
struct Cli {
    #[command(subcommand)]
    command: Option<Commands>,
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
                    let caps = core::registry::get_backend_capabilities(b.id)
                        .expect("backend disappeared");
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
            for p in core::list_profiles() {
                println!("{}  λ={} bits", p.id, p.lambda_bits);
            }
        }
        None => {
            println!("zkd {} — ready", core::version());
            println!("Try: `zkd backend-ls [-v]` or `zkd profile-ls`");
        }
    }
    Ok(())
}
