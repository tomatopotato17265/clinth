use std::io::{self, BufRead, Write};

use anyhow::Result;
use clap::Subcommand;

use crate::{collection, organization, project, report, version};

#[derive(Debug, Subcommand)]
pub enum CreateCommand {
    #[command(disable_help_flag = true)]
    Project,
    #[command(disable_help_flag = true)]
    Version,
    #[command(disable_help_flag = true)]
    Collection,
    #[command(disable_help_flag = true)]
    Organization,
    #[command(disable_help_flag = true)]
    Report,
}

pub fn run(command: CreateCommand, dry_run: bool) -> Result<()> {
    match command {
        CreateCommand::Project => project::run(dry_run),
        CreateCommand::Version => version::run(dry_run),
        CreateCommand::Collection => collection::run(dry_run),
        CreateCommand::Organization => organization::run(dry_run),
        CreateCommand::Report => report::run(dry_run),
    }
}

pub fn prompt(label: &str) -> Result<String> {
    let mut out = io::stdout();
    write!(out, "{label}")?;
    out.flush()?;
    let mut line = String::new();
    io::stdin().lock().read_line(&mut line)?;
    Ok(line.trim().to_string())
}

pub fn prompt_opt(label: &str) -> Result<Option<String>> {
    let value = prompt(label)?;
    Ok(if value.is_empty() { None } else { Some(value) })
}

pub fn prompt_bool(label: &str, default: bool) -> Result<bool> {
    Ok(match prompt(label)?.to_ascii_lowercase().as_str() {
        "y" | "yes" => true,
        "n" | "no" => false,
        _ => default,
    })
}

pub fn prompt_list(label: &str) -> Result<Vec<String>> {
    Ok(prompt(label)?
        .split(',')
        .map(str::trim)
        .filter(|s| !s.is_empty())
        .map(String::from)
        .collect())
}

pub fn emit(dry_run: bool, method: &str, url: &str, body: &serde_json::Value) -> bool {
    if dry_run {
        println!("{method} {url}");
        println!(
            "{}",
            serde_json::to_string_pretty(body).unwrap_or_else(|_| body.to_string())
        );
    }
    dry_run
}
