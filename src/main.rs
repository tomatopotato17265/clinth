mod auth;
mod collection;
mod modrinth;
mod organization;
mod project;
mod report;
mod version;

mod commands {
    pub mod create;
    pub mod discover;
    pub mod install;
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
    #[command(
        disable_help_flag = true,
        override_usage = "clinth install <content-type> <query> [--<loader>] [--<mc-version>]"
    )]
    Install {
        content_type: commands::discover::ContentType,
        #[arg(trailing_var_arg = true, allow_hyphen_values = true)]
        rest: Vec<String>,
    },
    #[command(disable_help_flag = true)]
    Create {
        #[arg(long, global = true)]
        dry_run: bool,
        #[command(subcommand)]
        command: commands::create::CreateCommand,
    },
}

fn main() -> Result<()> {
    match Cli::parse().command {
        Commands::Login => commands::login::run(),
        Commands::Whoami => commands::whoami::run(),
        Commands::Logout => commands::logout::run(),
        Commands::Discover { content_type, query } => commands::discover::run(content_type, &query),
        Commands::Install { content_type, rest } => commands::install::run(content_type, &rest),
        Commands::Create { dry_run, command } => commands::create::run(command, dry_run),
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

pub(crate) fn hit_line(hit: &modrinth::SearchHit) -> String {
    format!(
        "{} by {} - {}",
        bold(&hit.title),
        bold(&hit.author),
        underline(&format!(
            "https://modrinth.com/{}/{}",
            hit.project_type, hit.slug
        ))
    )
}
