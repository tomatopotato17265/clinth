mod auth;
mod modrinth;

mod commands {
    pub mod discover;
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
    #[command(
        disable_help_flag = true,
        override_usage = "clinth discover <content-type> <query>"
    )]
    Discover {
        content_type: commands::discover::ContentType,
        query: Vec<String>,
    },
}

fn main() -> Result<()> {
    match Cli::parse().command {
        Commands::Login => commands::login::run(),
        Commands::Whoami => commands::whoami::run(),
        Commands::Logout => commands::logout::run(),
        Commands::Discover { content_type, query } => commands::discover::run(content_type, &query),
    }
}

pub(crate) fn bold(text: &str) -> String {
    if std::io::stdout().is_terminal() {
        format!("\x1b[1m{text}\x1b[0m")
    } else {
        text.to_string()
    }
}

pub(crate) fn underline(text: &str) -> String {
    if std::io::stdout().is_terminal() {
        format!("\x1b[4m{text}\x1b[0m")
    } else {
        text.to_string()
    }
}
