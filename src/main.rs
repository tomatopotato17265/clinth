mod auth;

mod commands {
    pub mod login;
    pub mod logout;
    pub mod whoami;
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
    #[command(disable_help_flag = true)]
    Whoami,
    #[command(disable_help_flag = true)]
    Logout,
}

fn main() -> Result<()> {
    match Cli::parse().command {
        Commands::Login => commands::login::run(),
        Commands::Whoami => commands::whoami::run(),
        Commands::Logout => commands::logout::run(),
    }
}

pub(crate) fn bold(text: &str) -> String {
    if std::io::stdout().is_terminal() {
        format!("\x1b[1m{text}\x1b[0m")
    } else {
        text.to_string()
    }
}
