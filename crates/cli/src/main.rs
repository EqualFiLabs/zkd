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
    BackendLs,
    /// List available profiles
    ProfileLs,
}

fn main() -> Result<()> {
    let cli = Cli::parse();
    match cli.command {
        Some(Commands::BackendLs) => {
            for b in core::list_backends() {
                println!("{}  recursion={}", b.id, b.recursion);
            }
        }
        Some(Commands::ProfileLs) => {
            for p in core::list_profiles() {
                println!("{}  λ={} bits", p.id, p.lambda_bits);
            }
        }
        None => {
            println!("zkd {} — ready", core::version());
            println!("Try: `zkd backend-ls` or `zkd profile-ls`");
        }
    }
    Ok(())
}
