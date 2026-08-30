use std::io::{self, Write};

use anyhow::Result;

use crate::auth;

pub fn run() -> Result<()> {
    let Some(record) = auth::load()? else {
        println!("You are not logged in.");
        return Ok(());
    };

    print!(
        "Are you sure you want to log out from {}? [Y/n] ",
        record.username
    );
    io::stdout().flush()?;

    let mut answer = String::new();
    io::stdin().read_line(&mut answer)?;

    if matches!(answer.trim().to_ascii_lowercase().as_str(), "n" | "no") {
        println!("Logout canceled.");
        return Ok(());
    }

    auth::delete()?;
    println!("Logged out from {}.", record.username);
    Ok(())
}
