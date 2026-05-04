use std::io::{self, Write};
use std::path::{Path, PathBuf};

use anyhow::Result;
use clap::{Parser, Subcommand};
use gc_core::scan::UnknownGenre;
use gc_core::Taxonomy;

#[derive(Parser)]
#[command(name = "genre-centralizer")]
#[command(about = "Rewrite audio genre tags to canonical genres")]
struct Cli {
    #[command(subcommand)]
    command: Commands,
}

#[derive(Subcommand)]
enum Commands {
    /// Scan a directory for MP3 files and propose genre mappings
    Scan {
        /// Directory to scan
        directory: PathBuf,
        /// Actually write the proposed genre changes
        #[arg(long)]
        apply: bool,
    },
    /// Taxonomy management commands
    Taxonomy {
        #[command(subcommand)]
        command: TaxonomyCommands,
    },
}

#[derive(Subcommand)]
enum TaxonomyCommands {
    /// Validate the taxonomy for ambiguities
    Validate,
}

fn print_scan_header() {
    println!(
        "  {:<50} | {:<20} | Proposal",
        "File", "Current Genre"
    );
    println!(
        "  {:<50} | {:<20} | {}",
        "-".repeat(50),
        "-".repeat(20),
        "-".repeat(20)
    );
}

fn resolve_unknowns(
    unknowns: &[UnknownGenre],
    taxonomy: &mut Taxonomy,
) -> Result<Vec<(String, String)>> {
    println!("\n--- Unknown Genres ---\n");

    let mut resolved = Vec::new();
    let stdin = io::stdin();

    for unknown in unknowns {
        let file_count = unknown.files.len();
        println!(
            "  \"{}\" ({} file{})",
            unknown.genre,
            file_count,
            if file_count == 1 { "" } else { "s" }
        );

        let suggestions = taxonomy.suggest_matches(&unknown.genre, 3);

        if !suggestions.is_empty() {
            print!("  Suggestions: ");
            for (i, (name, _dist)) in suggestions.iter().enumerate() {
                if i > 0 {
                    print!(", ");
                }
                print!("[{}] {}", i + 1, name);
            }
            println!();
        }

        print!("  Map to (number, genre name, or Enter to skip): ");
        io::stdout().flush()?;

        let mut input = String::new();
        stdin.read_line(&mut input)?;
        let input = input.trim();

        if input.is_empty() {
            println!("  Skipped.\n");
            continue;
        }

        let canonical = if let Ok(num) = input.parse::<usize>() {
            if num >= 1 && num <= suggestions.len() {
                suggestions[num - 1].0.clone()
            } else {
                println!("  Invalid selection, skipped.\n");
                continue;
            }
        } else {
            input.to_string()
        };

        match taxonomy.add_alias(&unknown.genre, &canonical) {
            Ok(()) => {
                resolved.push((unknown.genre.clone(), canonical.clone()));
                println!("  Mapped \"{}\" -> \"{}\"\n", unknown.genre, canonical);
            }
            Err(e) => {
                println!("  Error: {}. Skipped.\n", e);
            }
        }
    }

    Ok(resolved)
}

fn apply_resolved_unknowns(
    unknowns: &[UnknownGenre],
    _taxonomy: &Taxonomy,
    resolved: &[(String, String)],
) -> Result<usize> {
    let mut written = 0;

    for unknown in unknowns {
        let canonical = resolved
            .iter()
            .find(|(alias, _)| alias == &unknown.genre)
            .map(|(_, c)| c.as_str());

        let Some(canonical) = canonical else {
            continue;
        };

        for file_path in &unknown.files {
            let path = Path::new(file_path);
            gc_core::tagio::write_canonical_and_preserve(path, canonical, &unknown.genre)?;
            println!(
                "  {:<50} | {:<20} | WROTE -> {}",
                file_path, unknown.genre, canonical
            );
            written += 1;
        }
    }

    Ok(written)
}

fn user_taxonomy_path() -> Option<PathBuf> {
    dirs::config_dir().map(|d| d.join("genre-centralizer").join("taxonomy.yaml"))
}

fn main() -> Result<()> {
    let cli = Cli::parse();
    let mut taxonomy = gc_core::Taxonomy::load_default()?;

    if let Some(user_path) = user_taxonomy_path() {
        if user_path.exists() {
            let yaml = std::fs::read_to_string(&user_path)?;
            taxonomy.merge_user_overrides(&yaml)?;
        }
    }

    match cli.command {
        Commands::Scan { directory, apply } => {
            if !directory.is_dir() {
                anyhow::bail!("Not a directory: {}", directory.display());
            }

            println!("Scanning: {}", directory.display());
            print_scan_header();

            let result = if apply {
                gc_core::scan::scan_and_apply(&directory, &taxonomy)?
            } else {
                gc_core::scan::scan_directory(&directory, &taxonomy)?
            };

            if apply && !result.unknown_genres.is_empty() {
                let resolved = resolve_unknowns(&result.unknown_genres, &mut taxonomy)?;

                if !resolved.is_empty() {
                    println!("--- Applying resolved unknowns ---\n");
                    let written =
                        apply_resolved_unknowns(&result.unknown_genres, &taxonomy, &resolved)?;
                    println!("\nWrote {} additional file(s).", written);

                    let all_aliases: Vec<(String, String)> = taxonomy
                        .user_added_aliases()
                        .to_vec();

                    if !all_aliases.is_empty() {
                        if let Some(user_path) = user_taxonomy_path() {
                            if user_path.exists() {
                                let existing = std::fs::read_to_string(&user_path)?;
                                let existing_file: gc_core::taxonomy::TaxonomyFile =
                                    serde_yaml::from_str(&existing)?;
                                let mut merged = all_aliases.clone();
                                for genre in &existing_file.canonical_genres {
                                    for alias in &genre.aliases {
                                        if !merged.iter().any(|(a, _)| a == alias) {
                                            merged.push((alias.clone(), genre.name.clone()));
                                        }
                                    }
                                }
                                gc_core::taxonomy::save_user_taxonomy(&merged, &user_path)?;
                            } else {
                                gc_core::taxonomy::save_user_taxonomy(&all_aliases, &user_path)?;
                            }
                            println!("Saved user taxonomy to {}", user_path.display());
                        }
                    }
                }
            }
        }
        Commands::Taxonomy { command } => match command {
            TaxonomyCommands::Validate => {
                println!(
                    "Taxonomy is valid. {} entries loaded.",
                    taxonomy.lookup().len()
                );
            }
        },
    }

    Ok(())
}
