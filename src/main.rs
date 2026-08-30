mod commands {
    pub mod login;
}

use anyhow::Result;
use clap::{Parser, Subcommand};

#[derive(Debug, Parser)]
#[command(name = "clinth", version, about)]
struct Cli {
    #[command(subcommand)]
    command: Commands,
}

#[derive(Debug, Subcommand)]
enum Commands {
    #[command(disable_help_flag = true)]
    Login,
}

fn main() -> Result<()> {
    match Cli::parse().command {
        Commands::Login => commands::login::run(),
    }
}
