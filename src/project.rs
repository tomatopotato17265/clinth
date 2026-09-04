use anyhow::{Result, bail};
use serde_json::json;

use crate::commands::create::{emit, prompt, prompt_bool, prompt_list};
use crate::modrinth;

pub fn run(dry_run: bool, _token: &str) -> Result<()> {
    let name = prompt("Name: ")?;
    let slug = prompt("Slug: ")?;
    let summary = prompt("Summary: ")?;
    let description = prompt("Description: ")?;
    let license_id = prompt("License ID (SPDX, e.g. MIT): ")?;

    let categories = loop {
        let list = prompt_list("Categories (comma-separated, max 3): ")?;
        if list.len() <= 3 {
            break list;
        }
        println!("At most 3 categories.");
    };

    let is_draft = prompt_bool("Draft? [Y/n]: ", true)?;

    let data = json!({
        "name": name,
        "slug": slug,
        "summary": summary,
        "description": description,
        "license_id": license_id,
        "categories": categories,
        "initial_versions": [],
        "is_draft": is_draft,
    });

    let url = format!("{}/project", modrinth::api("v3"));
    if emit(dry_run, "POST", &url, &data) {
        return Ok(());
    }

    bail!("not implemented")
}
