use anyhow::{Context, Result, bail};
use keyring::Entry;
use serde::{Deserialize, Serialize};

use crate::modrinth::{USER_AGENT, api};

const KEYRING_SERVICE: &str = "clinth";
const KEYRING_ACCOUNT: &str = "modrinth";

#[derive(Debug, Serialize, Deserialize)]
pub struct TokenRecord {
    pub access_token: String,
    pub token_type: String,
    pub expires_at: u64,
    pub username: String,
}

pub fn save(record: &TokenRecord) -> Result<()> {
    let json = serde_json::to_string(record)?;
    Entry::new(KEYRING_SERVICE, KEYRING_ACCOUNT)
        .context("failed to open the OS keyring")?
        .set_password(&json)
        .context("failed to write the credential to the OS keyring")
}

pub fn delete() -> Result<()> {
    let entry =
        Entry::new(KEYRING_SERVICE, KEYRING_ACCOUNT).context("failed to open the OS keyring")?;
    match entry.delete_credential() {
        Ok(()) | Err(keyring::Error::NoEntry) => Ok(()),
        Err(e) => Err(e).context("failed to delete the credential from the OS keyring"),
    }
}

pub fn token() -> Result<String> {
    match load()? {
        Some(record) => Ok(record.access_token),
        None => bail!("not logged in \u{2014} run `clinth login`"),
    }
}

pub fn load() -> Result<Option<TokenRecord>> {
    let entry = Entry::new(KEYRING_SERVICE, KEYRING_ACCOUNT).context("failed to open the OS keyring")?;
    match entry.get_password() {
        Ok(json) => Ok(Some(
            serde_json::from_str(&json).context("stored credential is not valid JSON")?,
        )),
        Err(keyring::Error::NoEntry) => Ok(None),
        Err(e) => Err(e).context("failed to read the credential from the OS keyring"),
    }
}

#[derive(Debug, Deserialize)]
struct User {
    username: String,
}

pub fn fetch_username(access_token: &str) -> Result<String> {
    let resp = reqwest::blocking::Client::new()
        .get(format!("{}/user", api("v2")))
        .header("Authorization", access_token)
        .header("User-Agent", USER_AGENT)
        .send()
        .context("failed to reach the Modrinth API")?;

    let status = resp.status();
    let body = resp.text().unwrap_or_default();
    if !status.is_success() {
        bail!("could not fetch your Modrinth profile ({status}): {body}");
    }
    let user: User = serde_json::from_str(&body)
        .with_context(|| format!("unexpected /v2/user response: {body}"))?;
    Ok(user.username)
}
