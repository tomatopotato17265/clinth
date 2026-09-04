use anyhow::Result;
use serde_json::json;

use crate::commands::create::{emit, prompt};
use crate::modrinth;

pub fn run(dry_run: bool, token: &str) -> Result<()> {
    let report_type = prompt(
        "Report type (spam / copyright / inappropriate / malicious / name-squatting / other): ",
    )?;

    let item_type = loop {
        let value = prompt("Item type (project / version / user): ")?;
        match value.as_str() {
            "project" | "version" | "user" => break value,
            _ => println!("Enter one of: project, version, user."),
        }
    };

    let item_id = prompt("Item ID: ")?;
    let body_text = prompt("Body: ")?;

    let body = json!({
        "report_type": report_type,
        "item_type": item_type,
        "item_id": item_id,
        "body": body_text,
    });

    let url = format!("{}/report", modrinth::api("v2"));
    if emit(dry_run, "POST", &url, &body) {
        return Ok(());
    }

    let resp = modrinth::post_json("v2", "/report", token, &body)?;
    let id = resp["id"].as_str().unwrap_or_default();
    let thread = resp["thread_id"].as_str().unwrap_or_default();
    println!("Submitted report {id} (moderation thread {thread})");
    Ok(())
}
