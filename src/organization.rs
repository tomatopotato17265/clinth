use anyhow::Result;
use serde_json::json;

use crate::commands::create::{emit, prompt};
use crate::modrinth;

pub fn run(dry_run: bool, token: &str) -> Result<()> {
    let slug = prompt("Slug: ")?;
    let name = prompt("Name: ")?;
    let description = prompt("Description: ")?;

    let body = json!({
        "slug": slug,
        "name": name,
        "description": description,
    });

    let url = format!("{}/organization", modrinth::api("v3"));
    if emit(dry_run, "POST", &url, &body) {
        return Ok(());
    }

    let resp = modrinth::post_json("v3", "/organization", token, &body)?;
    let created = resp["slug"].as_str().unwrap_or(&slug);
    println!("Created organization: https://modrinth.com/organization/{created}");
    Ok(())
}
