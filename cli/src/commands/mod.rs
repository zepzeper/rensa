use clap::Parser;
use std::path::PathBuf;

#[derive(Parser)]
pub struct Check {
    #[arg(short, long, help = "Path to scan")]
    path: Option<PathBuf>,
    #[arg(short, long, help = "Output in JSON format")]
    json: bool,
}

impl Check {
    pub async fn run(&self) -> anyhow::Result<()> {
        let path = self.path.clone().unwrap_or_else(|| PathBuf::from("."));

        if !path.exists() {
            anyhow::bail!("Path does not exist: {}", path.display());
        }

        let report = super::scan::run_scan(&path).await?;

        if self.json {
            println!("{}", super::display::print_json(&report)?);
        } else {
            super::display::print_report(&report);
        }

        Ok(())
    }
}

#[derive(Parser)]
pub struct Ecosystems;

impl Ecosystems {
    pub fn run(&self) {
        println!("Supported ecosystems:");
        println!("  - composer");
        println!("  - npm (coming soon)");
        println!("  - cargo (coming soon)");
        println!("  - pypi (coming soon)");
    }
}
