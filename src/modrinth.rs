use std::path::PathBuf;

use anyhow::{Context, Result, bail};
use serde::Deserialize;

const DEFAULT_API_BASE: &str = "https://api.modrinth.com";

pub const USER_AGENT: &str = concat!(
    "clinth/",
    env!("CARGO_PKG_VERSION"),
    " (+https://github.com/tomatopotato17265/clinth)"
);

pub fn api(version: &str) -> String {
    let base = std::env::var("CLINTH_API_BASE").unwrap_or_else(|_| DEFAULT_API_BASE.to_string());
    format!("{base}/{version}")
}

fn read_json(resp: reqwest::blocking::Response) -> Result<serde_json::Value> {
    let status = resp.status();
    let text = resp.text().unwrap_or_default();
    if !status.is_success() {
        bail!("Modrinth API request failed ({status}): {text}");
    }
    serde_json::from_str(&text).with_context(|| format!("unexpected API response: {text}"))
}

pub fn post_json(
    version: &str,
    path: &str,
    token: &str,
    body: &serde_json::Value,
) -> Result<serde_json::Value> {
    let resp = reqwest::blocking::Client::new()
        .post(format!("{}{path}", api(version)))
        .header("Authorization", token)
        .header("User-Agent", USER_AGENT)
        .json(body)
        .send()
        .context("failed to reach the Modrinth API")?;
    read_json(resp)
}

pub fn post_multipart(
    version: &str,
    path: &str,
    token: &str,
    data: &serde_json::Value,
    file_parts: &[(String, PathBuf)],
) -> Result<serde_json::Value> {
    let mut form = reqwest::blocking::multipart::Form::new().text("data", data.to_string());
    for (field, file) in file_parts {
        let part = reqwest::blocking::multipart::Part::file(file)
            .with_context(|| format!("could not read {}", file.display()))?;
        form = form.part(field.clone(), part);
    }

    let resp = reqwest::blocking::Client::new()
        .post(format!("{}{path}", api(version)))
        .header("Authorization", token)
        .header("User-Agent", USER_AGENT)
        .multipart(form)
        .send()
        .context("failed to reach the Modrinth API")?;
    read_json(resp)
}

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

#[derive(Debug, Deserialize)]
pub struct Version {
    pub files: Vec<VersionFile>,
}

#[derive(Debug, Deserialize)]
pub struct VersionFile {
    pub url: String,
    pub filename: String,
    pub primary: bool,
}

fn json_array(values: &[String]) -> String {
    let inner = values
        .iter()
        .map(|v| format!("\"{v}\""))
        .collect::<Vec<_>>()
        .join(",");
    format!("[{inner}]")
}

pub fn fetch_versions(
    slug: &str,
    loaders: &[String],
    game_versions: &[String],
) -> Result<Vec<Version>> {
    let mut params: Vec<(&str, String)> = Vec::new();
    if !loaders.is_empty() {
        params.push(("loaders", json_array(loaders)));
    }
    if !game_versions.is_empty() {
        params.push(("game_versions", json_array(game_versions)));
    }

    let resp = reqwest::blocking::Client::new()
        .get(format!("{}/project/{slug}/version", api("v2")))
        .header("User-Agent", USER_AGENT)
        .query(&params)
        .send()
        .context("failed to reach the Modrinth API")?;

    let status = resp.status();
    let body = resp.text().unwrap_or_default();
    if !status.is_success() {
        bail!("could not list versions for {slug} ({status}): {body}");
    }
    serde_json::from_str(&body).with_context(|| format!("unexpected version-list response: {body}"))
}

pub fn download_bytes(url: &str) -> Result<Vec<u8>> {
    let resp = reqwest::blocking::Client::new()
        .get(url)
        .header("User-Agent", USER_AGENT)
        .send()
        .context("failed to download file")?;

    let status = resp.status();
    if !status.is_success() {
        bail!("download failed ({status}): {url}");
    }
    Ok(resp.bytes().context("failed to read downloaded file")?.to_vec())
}

pub fn search(project_type: &str, query: &str, limit: u32) -> Result<Vec<SearchHit>> {
    let facets = format!(r#"[["project_type:{project_type}"]]"#);
    let resp = reqwest::blocking::Client::new()
        .get(format!("{}/search", api("v2")))
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
