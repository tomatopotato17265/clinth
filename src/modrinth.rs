use anyhow::{Context, Result, bail};
use serde::Deserialize;

pub const API_BASE: &str = "https://api.modrinth.com";

pub const USER_AGENT: &str = concat!(
    "clinth/",
    env!("CARGO_PKG_VERSION"),
    " (+https://github.com/tomatopotato17265/clinth)"
);

#[derive(Debug, Deserialize)]
pub struct SearchHit {
    pub title: String,
    pub slug: String,
    pub author: String,
    pub project_type: String,
}

#[derive(Debug, Deserialize)]
struct SearchResponse {
    hits: Vec<SearchHit>,
}

pub fn search(project_type: &str, query: &str, limit: u32) -> Result<Vec<SearchHit>> {
    let facets = format!(r#"[["project_type:{project_type}"]]"#);
    let resp = reqwest::blocking::Client::new()
        .get(format!("{API_BASE}/v2/search"))
        .header("User-Agent", USER_AGENT)
        .query(&[
            ("query", query),
            ("facets", facets.as_str()),
            ("limit", &limit.to_string()),
        ])
        .send()
        .context("failed to reach the Modrinth API")?;

    let status = resp.status();
    let body = resp.text().unwrap_or_default();
    if !status.is_success() {
        bail!("Modrinth search failed ({status}): {body}");
    }
    let parsed: SearchResponse = serde_json::from_str(&body)
        .with_context(|| format!("unexpected /v2/search response: {body}"))?;
    Ok(parsed.hits)
}
