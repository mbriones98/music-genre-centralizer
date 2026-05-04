use std::path::Path;

use anyhow::{Context, Result};
use lofty::file::TaggedFileExt;
use lofty::tag::Accessor;

use crate::taxonomy::{Classification, Taxonomy};

pub struct ScanResult {
    pub found: usize,
    pub matched: usize,
    pub unknown_genres: Vec<UnknownGenre>,
}

pub struct UnknownGenre {
    pub genre: String,
    pub files: Vec<String>,
}

pub fn scan_directory(dir: &Path, taxonomy: &Taxonomy) -> Result<ScanResult> {
    let mut found = 0;
    let mut matched = 0;
    let mut unknown_map: std::collections::HashMap<String, Vec<String>> =
        std::collections::HashMap::new();

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
                unknown_map
                    .entry(String::new())
                    .or_default()
                    .push(path.display().to_string());
                continue;
            }
        };

        let current_genre = tag.genre().map(|g| g.to_string()).unwrap_or_default();

        match taxonomy.classify(&current_genre) {
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
                unknown_map
                    .entry(current_genre)
                    .or_default()
                    .push(path.display().to_string());
            }
            Classification::Empty => {
                println!("  {:<50} | {:<20} | SKIP (empty genre)", path.display(), "—");
                unknown_map
                    .entry(String::new())
                    .or_default()
                    .push(path.display().to_string());
            }
        }
    }

    let unknown_genres: Vec<UnknownGenre> = unknown_map
        .into_iter()
        .filter(|(genre, _)| !genre.is_empty())
        .map(|(genre, files)| UnknownGenre { genre, files })
        .collect();

    println!();
    println!(
        "Summary: {} MP3s found, {} matched, {} unknown/skipped",
        found,
        matched,
        found - matched
    );

    Ok(ScanResult {
        found,
        matched,
        unknown_genres,
    })
}

pub fn scan_and_apply(dir: &Path, taxonomy: &Taxonomy) -> Result<ScanResult> {
    let mut found = 0;
    let mut matched = 0;
    let mut applied = 0;
    let mut unknown_map: std::collections::HashMap<String, Vec<String>> =
        std::collections::HashMap::new();

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
                continue;
            }
        };

        let current_genre = tag.genre().map(|g| g.to_string()).unwrap_or_default();

        match taxonomy.classify(&current_genre) {
            Classification::AlreadyCanonical => {
                println!(
                    "  {:<50} | {:<20} | OK (already canonical)",
                    path.display(),
                    current_genre
                );
                matched += 1;
            }
            Classification::Mapped { canonical } => {
                crate::tagio::write_canonical_and_preserve(&path, &canonical, &current_genre)?;
                println!(
                    "  {:<50} | {:<20} | WROTE -> {}",
                    path.display(),
                    current_genre,
                    canonical
                );
                matched += 1;
                applied += 1;
            }
            Classification::Unknown => {
                println!(
                    "  {:<50} | {:<20} | UNKNOWN (skipped)",
                    path.display(),
                    current_genre
                );
                unknown_map
                    .entry(current_genre)
                    .or_default()
                    .push(path.display().to_string());
            }
            Classification::Empty => {
                println!("  {:<50} | {:<20} | SKIP (empty genre)", path.display(), "—");
            }
        }
    }

    let unknown_genres: Vec<UnknownGenre> = unknown_map
        .into_iter()
        .filter(|(genre, _)| !genre.is_empty())
        .map(|(genre, files)| UnknownGenre { genre, files })
        .collect();

    println!();
    println!(
        "Summary: {} MP3s found, {} matched ({} written), {} unknown/skipped",
        found,
        matched,
        applied,
        found - matched
    );

    Ok(ScanResult {
        found,
        matched,
        unknown_genres,
    })
}
