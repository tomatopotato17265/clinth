use anyhow::Result;
use clap::ValueEnum;

use crate::modrinth;

#[derive(Debug, Clone, Copy, ValueEnum)]
pub enum ContentType {
    Mod,
    Modpack,
    Resourcepack,
    Shader,
    Plugin,
    Datapack,
}

impl ContentType {
    pub fn facet(self) -> &'static str {
        match self {
            ContentType::Mod => "mod",
            ContentType::Modpack => "modpack",
            ContentType::Resourcepack => "resourcepack",
            ContentType::Shader => "shader",
            ContentType::Plugin => "plugin",
            ContentType::Datapack => "datapack",
        }
    }
}

pub fn run(content_type: ContentType, query: &[String]) -> Result<()> {
    let hits = modrinth::search(content_type.facet(), &query.join(" "), 20)?;
    if hits.is_empty() {
        println!("No results.");
        return Ok(());
    }
    for hit in hits {
        println!("{}", crate::hit_line(&hit));
    }
    Ok(())
}
