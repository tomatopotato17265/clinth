use anyhow::{Result, bail};

use crate::auth;

pub fn run() -> Result<()> {
    let Some(record) = auth::load()? else {
        bail!("not logged in \u{2014} run `clinth login` first");
    };
    let username = auth::fetch_username(&record.access_token)?;
    println!("{}", crate::bold(&format!("Logged in as {username}.")));
    Ok(())
}
