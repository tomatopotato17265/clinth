use anyhow::Result;
use serde_json::{Value, json};

use crate::commands::create::{emit, prompt, prompt_list, prompt_opt};
use crate::modrinth;

pub fn run(dry_run: bool, token: &str) -> Result<()> {
    let name = prompt("Name: ")?;
    let description = prompt_opt("Description (optional): ")?;
    let projects = prompt_list("Projects (comma-separated ids/slugs, optional): ")?;

    let mut body = json!({
        "name": name,
        "projects": projects,
    });
    if let Some(description) = description {
        body["description"] = Value::String(description);
    }

    let url = format!("{}/collection", modrinth::api("v3"));
    if emit(dry_run, "POST", &url, &body) {
        return Ok(());
    }

    let resp = modrinth::post_json("v3", "/collection", token, &body)?;
    let id = resp["id"].as_str().unwrap_or_default();
    println!("Created collection: https://modrinth.com/collection/{id}");
    Ok(())
}
