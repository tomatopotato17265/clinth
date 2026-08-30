use std::io::IsTerminal;
use std::time::{SystemTime, UNIX_EPOCH};

use anyhow::{Context, Result, anyhow, bail};
use keyring::Entry;
use serde::{Deserialize, Serialize};
use tiny_http::{Header, Request, Response, Server};

const CLIENT_ID: &str = "Lg8hvjeB";
const WORKER_TOKEN_URL: &str = "https://clinth.tomatopotato17265.workers.dev/token";
const REDIRECT_PORT: u16 = 7113;
const REDIRECT_URI: &str = "http://localhost:7113/callback";
const AUTHORIZE_URL: &str = "https://modrinth.com/auth/authorize";
const API_BASE: &str = "https://api.modrinth.com";
const SCOPES: &str = "USER_READ";

const USER_AGENT: &str = concat!(
    "clinth/",
    env!("CARGO_PKG_VERSION"),
    " (+https://github.com/tomatopotato17265/clinth)"
);

const KEYRING_SERVICE: &str = "clinth";
const KEYRING_ACCOUNT: &str = "modrinth";

pub fn run() -> Result<()> {
    let state = random_state()?;
    let url = authorize_url(&state)?;

    println!("Opening your browser to log in with Modrinth\u{2026}");
    println!("If it doesn't open automatically, visit:\n  {url}\n");
    let _ = webbrowser::open(&url);

    let code = wait_for_code(&state)?;
    let token = exchange_code(&code)?;
    let username = fetch_username(&token.access_token)?;

    save_token(&TokenRecord {
        access_token: token.access_token,
        token_type: token.token_type,
        expires_at: now_unix().saturating_add(token.expires_in),
        username: username.clone(),
    })?;

    println!("{}", bold(&format!("Logged in as {username}.")));
    Ok(())
}

fn random_state() -> Result<String> {
    let mut bytes = [0u8; 32];
    getrandom::getrandom(&mut bytes)
        .map_err(|e| anyhow!("failed to generate a random state value: {e}"))?;
    Ok(bytes.iter().map(|b| format!("{b:02x}")).collect())
}

fn authorize_url(state: &str) -> Result<String> {
    let mut url = reqwest::Url::parse(AUTHORIZE_URL)?;
    url.query_pairs_mut()
        .append_pair("response_type", "code")
        .append_pair("client_id", CLIENT_ID)
        .append_pair("redirect_uri", REDIRECT_URI)
        .append_pair("scope", SCOPES)
        .append_pair("state", state);
    Ok(url.to_string())
}

fn wait_for_code(expected: &str) -> Result<String> {
    let addr = format!("127.0.0.1:{REDIRECT_PORT}");
    let server = Server::http(&addr).map_err(|e| {
        anyhow!(
            "could not bind {addr}: {e}. Is another process using port {REDIRECT_PORT}? \
             The redirect URI is registered with Modrinth and cannot change."
        )
    })?;

    for request in server.incoming_requests() {
        let Some(query) = request.url().split_once('?').map(|(_, q)| q.to_owned()) else {
            respond(request, 404, "Not found.");
            continue;
        };

        let (mut code, mut state, mut oauth_error) = (None, None, None);
        if let Ok(parsed) = reqwest::Url::parse(&format!("http://localhost/?{query}")) {
            for (k, v) in parsed.query_pairs() {
                match k.as_ref() {
                    "code" => code = Some(v.into_owned()),
                    "state" => state = Some(v.into_owned()),
                    "error" => oauth_error = Some(v.into_owned()),
                    _ => {}
                }
            }
        }

        if let Some(err) = oauth_error {
            respond(request, 400, "Authorization failed. You can close this tab.");
            bail!("Modrinth returned an OAuth error: {err}");
        }

        match (code, state) {
            (Some(code), Some(state)) if state == expected => {
                respond(
                    request,
                    200,
                    "clinth is now logged in. You can close this tab and return to the terminal.",
                );
                return Ok(code);
            }
            (_, Some(_)) => {
                respond(request, 400, "State mismatch. You can close this tab.");
                bail!("OAuth state mismatch \u{2014} aborting for safety");
            }
            _ => {
                respond(
                    request,
                    400,
                    "Missing authorization code. You can close this tab.",
                );
                bail!("the OAuth callback did not include an authorization code");
            }
        }
    }

    bail!("the local server stopped before receiving the OAuth callback")
}

fn respond(request: Request, status: u16, message: &str) {
    let html = format!(
        "<!doctype html><meta charset=\"utf-8\"><title>clinth</title>\
         <body style=\"font-family:system-ui,sans-serif;margin:3rem;font-size:1rem\">\
         <p>{message}</p></body>"
    );
    let header =
        Header::from_bytes(&b"Content-Type"[..], &b"text/html; charset=utf-8"[..]).expect("header");
    let _ = request.respond(
        Response::from_string(html)
            .with_status_code(status)
            .with_header(header),
    );
}

#[derive(Debug, Deserialize)]
struct TokenResponse {
    access_token: String,
    #[serde(default = "default_token_type")]
    token_type: String,
    #[serde(default)]
    expires_in: u64,
}

fn default_token_type() -> String {
    "Bearer".to_string()
}

fn exchange_code(code: &str) -> Result<TokenResponse> {
    let resp = reqwest::blocking::Client::new()
        .post(WORKER_TOKEN_URL)
        .json(&serde_json::json!({ "code": code, "redirect_uri": REDIRECT_URI }))
        .send()
        .context("failed to reach the token-exchange Worker")?;

    let status = resp.status();
    let body = resp.text().unwrap_or_default();
    if !status.is_success() {
        bail!("token exchange failed ({status}): {body}");
    }
    serde_json::from_str(&body).with_context(|| format!("unexpected token response: {body}"))
}

#[derive(Debug, Deserialize)]
struct User {
    username: String,
}

fn fetch_username(access_token: &str) -> Result<String> {
    let resp = reqwest::blocking::Client::new()
        .get(format!("{API_BASE}/v2/user"))
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

#[derive(Debug, Serialize, Deserialize)]
struct TokenRecord {
    access_token: String,
    token_type: String,
    expires_at: u64,
    username: String,
}

fn save_token(record: &TokenRecord) -> Result<()> {
    let json = serde_json::to_string(record)?;
    Entry::new(KEYRING_SERVICE, KEYRING_ACCOUNT)
        .context("failed to open the OS keyring")?
        .set_password(&json)
        .context("failed to write the credential to the OS keyring")
}

fn bold(text: &str) -> String {
    if std::io::stdout().is_terminal() {
        format!("\x1b[1m{text}\x1b[0m")
    } else {
        text.to_string()
    }
}

fn now_unix() -> u64 {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map(|d| d.as_secs())
        .unwrap_or(0)
}
