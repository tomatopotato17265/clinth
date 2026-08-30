mod auth;

mod commands {
    pub mod login;
}

use std::io::IsTerminal;

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

pub(crate) fn bold(text: &str) -> String {
    if std::io::stdout().is_terminal() {
        format!("\x1b[1m{text}\x1b[0m")
    } else {
        text.to_string()
    }
}
