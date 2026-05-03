use std::collections::HashMap;
use std::path::Path;

use anyhow::{Context, Result};
use lofty::file::TaggedFileExt;
use lofty::tag::Accessor;

#[derive(Debug, PartialEq)]
enum Classification {
    AlreadyCanonical,
    Mapped { canonical: String },
    Unknown,
    Empty,
}

fn normalize(genre: &str) -> String {
    genre
        .chars()
        .filter(|c| c.is_alphanumeric())
        .collect::<String>()
        .to_lowercase()
}

fn build_taxonomy() -> HashMap<String, String> {
    let mappings: &[(&str, &str)] = &[
        // canonical → self
        ("Hip Hop", "Hip Hop"),
        ("Rock", "Rock"),
        ("Electronic", "Electronic"),
        ("Jazz", "Jazz"),
        ("Classical", "Classical"),
        ("R&B", "R&B"),
        ("Pop", "Pop"),
        ("Country", "Country"),
        ("Metal", "Metal"),
        ("Folk", "Folk"),
        // Hip Hop aliases
        ("Rap", "Hip Hop"),
        ("Cloud Rap", "Hip Hop"),
        ("Trap", "Hip Hop"),
        ("Drill", "Hip Hop"),
        ("Boom Bap", "Hip Hop"),
        // Rock aliases
        ("Shoegaze", "Rock"),
        ("Indie Rock", "Rock"),
        ("Post-Rock", "Rock"),
        ("Alternative Rock", "Rock"),
        ("Punk", "Rock"),
        ("Grunge", "Rock"),
        // Electronic aliases
        ("House", "Electronic"),
        ("Tech House", "Electronic"),
        ("Techno", "Electronic"),
        ("Drum and Bass", "Electronic"),
        ("Dubstep", "Electronic"),
        ("Ambient", "Electronic"),
        ("IDM", "Electronic"),
        ("Trance", "Electronic"),
        // Metal aliases
        ("Death Metal", "Metal"),
        ("Black Metal", "Metal"),
        ("Thrash Metal", "Metal"),
        ("Doom Metal", "Metal"),
    ];

    let mut map = HashMap::with_capacity(mappings.len());
    for (alias, canonical) in mappings {
        map.insert(normalize(alias), canonical.to_string());
    }
    map
}

fn classify(genre: &str, taxonomy: &HashMap<String, String>) -> Classification {
    if genre.is_empty() {
        return Classification::Empty;
    }

    let normalized = normalize(genre);

    match taxonomy.get(&normalized) {
        Some(canonical) if canonical == genre => Classification::AlreadyCanonical,
        Some(canonical) => Classification::Mapped {
            canonical: canonical.clone(),
        },
        None => Classification::Unknown,
    }
}

fn scan_directory(dir: &Path, taxonomy: &HashMap<String, String>) -> Result<()> {
    let mut found = 0;
    let mut matched = 0;
    let mut unknown = 0;

    let entries = std::fs::read_dir(dir)
        .with_context(|| format!("Failed to read directory: {}", dir.display()))?;

    for entry in entries {
        let entry = entry?;
        let path = entry.path();

        let is_mp3 = path
            .extension()
            .and_then(|e| e.to_str())
            .is_some_and(|ext| ext.eq_ignore_ascii_case("mp3"));

        if !is_mp3 {
            continue;
        }

        found += 1;

        let tagged_file = lofty::read_from_path(&path)
            .with_context(|| format!("Failed to read tags: {}", path.display()))?;

        let tag = match tagged_file.primary_tag().or(tagged_file.first_tag()) {
            Some(t) => t,
            None => {
                println!("  {:<50} | {:<20} | SKIP (no tags)", path.display(), "—");
                unknown += 1;
                continue;
            }
        };

        let current_genre = tag.genre().map(|g| g.to_string()).unwrap_or_default();

        match classify(&current_genre, taxonomy) {
            Classification::AlreadyCanonical => {
                println!(
                    "  {:<50} | {:<20} | OK (already canonical)",
                    path.display(),
                    current_genre
                );
                matched += 1;
            }
            Classification::Mapped { canonical } => {
                println!(
                    "  {:<50} | {:<20} | -> {}",
                    path.display(),
                    current_genre,
                    canonical
                );
                matched += 1;
            }
            Classification::Unknown => {
                println!(
                    "  {:<50} | {:<20} | UNKNOWN",
                    path.display(),
                    current_genre
                );
                unknown += 1;
            }
            Classification::Empty => {
                println!("  {:<50} | {:<20} | SKIP (empty genre)", path.display(), "—");
                unknown += 1;
            }
        }
    }

    println!();
    println!(
        "Summary: {} MP3s found, {} matched, {} unknown/skipped",
        found, matched, unknown
    );

    Ok(())
}

fn main() -> Result<()> {
    let args: Vec<String> = std::env::args().collect();

    if args.len() < 2 {
        eprintln!("Usage: genre-centralizer <directory>");
        eprintln!();
        eprintln!("Scans a directory for MP3 files and proposes canonical genre mappings.");
        std::process::exit(1);
    }

    let dir = Path::new(&args[1]);

    if !dir.is_dir() {
        anyhow::bail!("Not a directory: {}", dir.display());
    }

    let taxonomy = build_taxonomy();

    println!("Scanning: {}", dir.display());
    println!("  {:<50} | {:<20} | Proposal", "File", "Current Genre");
    println!(
        "  {:<50} | {:<20} | {}",
        "-".repeat(50),
        "-".repeat(20),
        "-".repeat(20)
    );

    scan_directory(dir, &taxonomy)?;

    Ok(())
}

#[cfg(test)]
mod tests;
