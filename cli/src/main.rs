mod commands;
mod scan;
mod display;

use clap::Parser;
use anyhow::Result;
use std::path::PathBuf;

#[derive(Parser)]
#[command(name = "rensa")]
#[command(about = "Dependency vulnerability scanner", long_about = None)]
struct Cli {
    #[command(subcommand)]
    command: Commands,
}

#[derive(Parser)]
enum Commands {
    #[command(about = "Check for vulnerabilities and updates")]
    Check(commands::Check),

    #[command(about = "List supported ecosystems")]
    Ecosystems(commands::Ecosystems),
}

#[tokio::main]
async fn main() -> Result<()> {
    let cli = Cli::parse();

    match cli.command {
        Commands::Check(cmd) => {
            cmd.run().await?;
        }
        Commands::Ecosystems(cmd) => {
            cmd.run();
        }
    }

    Ok(())
}
