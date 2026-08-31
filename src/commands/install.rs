use std::io::{self, BufRead, Write};
use std::path::{Path, PathBuf};

use anyhow::{Result, bail};

use crate::commands::discover::ContentType;
use crate::modrinth;

pub fn run(content_type: ContentType, rest: &[String]) -> Result<()> {
    let mut loaders: Vec<String> = Vec::new();
    let mut game_versions: Vec<String> = Vec::new();
    let mut query_words: Vec<&str> = Vec::new();

    for token in rest {
        if let Some(flag) = token.strip_prefix("--") {
            if flag == "help" {
                continue;
            }
            if flag.starts_with(|c: char| c.is_ascii_digit()) {
                game_versions.push(flag.to_string());
            } else {
                loaders.push(flag.to_string());
            }
        } else if token == "-h" {
            continue;
        } else {
            query_words.push(token);
        }
    }

    let query = query_words.join(" ");
    let hits = modrinth::search(content_type.facet(), &query, 20)?;
    if hits.is_empty() {
        println!("No results.");
        return Ok(());
    }

    for (i, hit) in hits.iter().enumerate() {
        println!("{:>2}. {}", i + 1, crate::hit_line(hit));
    }

    let selection = prompt_selection(content_type.facet(), hits.len())?;
    let Some(selection) = selection else {
        println!("Nothing selected.");
        return Ok(());
    };

    let mut failed = false;
    for index in selection {
        let hit = &hits[index - 1];
        if let Err(e) = install_one(hit, &loaders, &game_versions) {
            eprintln!("{}: {e}", hit.title);
            failed = true;
        }
    }

    if failed {
        bail!("one or more downloads failed");
    }
    Ok(())
}

fn prompt_selection(noun: &str, count: usize) -> Result<Option<Vec<usize>>> {
    let mut stdin = io::stdin().lock();
    loop {
        print!("List the {noun}(s) you want to install (use commas for multiple): ");
        io::stdout().flush()?;

        let mut line = String::new();
        if stdin.read_line(&mut line)? == 0 {
            bail!("no selection given");
        }

        let trimmed = line.trim();
        if trimmed.is_empty() {
            return Ok(None);
        }

        let mut chosen: Vec<usize> = Vec::new();
        let mut bad: Option<String> = None;
        for part in trimmed.split(',') {
            let part = part.trim();
            match part.parse::<usize>() {
                Ok(n) if (1..=count).contains(&n) => {
                    if !chosen.contains(&n) {
                        chosen.push(n);
                    }
                }
                _ => {
                    bad = Some(part.to_string());
                    break;
                }
            }
        }

        match bad {
            Some(part) => println!("\"{part}\" is not a number between 1 and {count}."),
            None => return Ok(Some(chosen)),
        }
    }
}

fn install_one(hit: &modrinth::SearchHit, loaders: &[String], game_versions: &[String]) -> Result<()> {
    let versions = modrinth::fetch_versions(&hit.slug, loaders, game_versions)?;
    let Some(version) = versions.first() else {
        bail!("no matching version");
    };
    let Some(file) = version.files.iter().find(|f| f.primary).or_else(|| version.files.first()) else {
        bail!("version has no files");
    };

    let bytes = modrinth::download_bytes(&file.url)?;
    let path = unique_path(Path::new("."), &file.filename);
    std::fs::write(&path, bytes)?;
    println!("Downloaded {}", path.display());
    Ok(())
}

fn unique_path(dir: &Path, filename: &str) -> PathBuf {
    let first = dir.join(filename);
    if !first.exists() {
        return first;
    }

    let path = Path::new(filename);
    let stem = path.file_stem().and_then(|s| s.to_str()).unwrap_or(filename);
    let ext = path.extension().and_then(|e| e.to_str());

    for n in 2.. {
        let candidate = match ext {
            Some(ext) => format!("{stem} ({n}).{ext}"),
            None => format!("{stem} ({n})"),
        };
        let candidate = dir.join(candidate);
        if !candidate.exists() {
            return candidate;
        }
    }
    unreachable!()
}
