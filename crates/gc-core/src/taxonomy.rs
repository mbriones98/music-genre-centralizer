use std::collections::HashMap;

use anyhow::{Result, bail};
use serde::{Deserialize, Serialize};

#[derive(Debug, PartialEq, Clone)]
pub enum Classification {
    AlreadyCanonical,
    Mapped { canonical: String },
    Unknown,
    Empty,
}

#[derive(Debug, Deserialize, Serialize, Clone)]
pub struct TaxonomyFile {
    pub version: u32,
    pub canonical_genres: Vec<CanonicalGenre>,
    #[serde(default)]
    pub ignored: Vec<String>,
}

#[derive(Debug, Deserialize, Serialize, Clone)]
pub struct CanonicalGenre {
    pub name: String,
    pub aliases: Vec<String>,
}

pub fn normalize(genre: &str) -> String {
    genre
        .chars()
        .filter(|c| c.is_alphanumeric())
        .collect::<String>()
        .to_lowercase()
}

pub struct Taxonomy {
    lookup: HashMap<String, String>,
    canonical_names: Vec<String>,
    user_added: Vec<(String, String)>,
}

impl Taxonomy {
    pub fn from_yaml(yaml_str: &str) -> Result<Self> {
        let file: TaxonomyFile = serde_yaml::from_str(yaml_str)?;
        let mut lookup = HashMap::new();
        let mut canonical_names = Vec::new();

        for genre in &file.canonical_genres {
            canonical_names.push(genre.name.clone());

            let key = normalize(&genre.name);
            if let Some(existing) = lookup.get(&key) {
                if existing != &genre.name {
                    bail!(
                        "Ambiguous normalized key '{}': canonical '{}' conflicts with '{}'",
                        key,
                        genre.name,
                        existing
                    );
                }
            }
            lookup.insert(key, genre.name.clone());

            for alias in &genre.aliases {
                let alias_key = normalize(alias);
                if let Some(existing) = lookup.get(&alias_key) {
                    if existing != &genre.name {
                        bail!(
                            "Ambiguous alias '{}' (normalized: '{}'): maps to both '{}' and '{}'",
                            alias,
                            alias_key,
                            existing,
                            genre.name
                        );
                    }
                }
                lookup.insert(alias_key, genre.name.clone());
            }
        }

        Ok(Taxonomy {
            lookup,
            canonical_names,
            user_added: Vec::new(),
        })
    }

    pub fn load_default() -> Result<Self> {
        Self::from_yaml(include_str!("../../../taxonomy/default.yaml"))
    }

    pub fn classify(&self, genre: &str) -> Classification {
        if genre.is_empty() {
            return Classification::Empty;
        }

        let normalized = normalize(genre);

        match self.lookup.get(&normalized) {
            Some(canonical) if canonical == genre => Classification::AlreadyCanonical,
            Some(canonical) => Classification::Mapped {
                canonical: canonical.clone(),
            },
            None => Classification::Unknown,
        }
    }

    pub fn lookup(&self) -> &HashMap<String, String> {
        &self.lookup
    }

    pub fn canonical_names(&self) -> &[String] {
        &self.canonical_names
    }

    /// Returns top fuzzy-match suggestions for an unknown genre.
    /// Each result is (canonical_name, edit_distance), sorted by distance.
    pub fn suggest_matches(&self, genre: &str, max: usize) -> Vec<(String, usize)> {
        let normalized = normalize(genre);
        let mut scores: Vec<(String, usize)> = self
            .canonical_names
            .iter()
            .map(|name| {
                let dist = strsim::damerau_levenshtein(&normalized, &normalize(name));
                (name.clone(), dist)
            })
            .filter(|(_, dist)| *dist <= 3)
            .collect();

        scores.sort_by_key(|(_, dist)| *dist);
        scores.truncate(max);
        scores
    }

    pub fn add_alias(&mut self, alias: &str, canonical: &str) -> Result<()> {
        if !self.canonical_names.contains(&canonical.to_string()) {
            bail!("'{}' is not a canonical genre", canonical);
        }

        let key = normalize(alias);
        if let Some(existing) = self.lookup.get(&key) {
            if existing != canonical {
                bail!(
                    "Alias '{}' already maps to '{}', cannot remap to '{}'",
                    alias,
                    existing,
                    canonical
                );
            }
            return Ok(());
        }

        self.lookup.insert(key, canonical.to_string());
        self.user_added.push((alias.to_string(), canonical.to_string()));
        Ok(())
    }

    pub fn user_added_aliases(&self) -> &[(String, String)] {
        &self.user_added
    }

    /// Merge aliases from a user taxonomy file into the in-memory lookup.
    pub fn merge_user_overrides(&mut self, yaml_str: &str) -> Result<()> {
        let file: TaxonomyFile = serde_yaml::from_str(yaml_str)?;

        for genre in &file.canonical_genres {
            for alias in &genre.aliases {
                let key = normalize(alias);
                if let Some(existing) = self.lookup.get(&key) {
                    if existing != &genre.name {
                        bail!(
                            "User taxonomy conflict: alias '{}' maps to '{}' in defaults but '{}' in user file",
                            alias,
                            existing,
                            genre.name
                        );
                    }
                }
                self.lookup.insert(key, genre.name.clone());
            }
        }

        Ok(())
    }
}

pub fn save_user_taxonomy(aliases: &[(String, String)], path: &std::path::Path) -> Result<()> {
    let mut genres_map: HashMap<String, Vec<String>> = HashMap::new();
    for (alias, canonical) in aliases {
        genres_map
            .entry(canonical.clone())
            .or_default()
            .push(alias.clone());
    }

    let canonical_genres: Vec<CanonicalGenre> = genres_map
        .into_iter()
        .map(|(name, aliases)| CanonicalGenre { name, aliases })
        .collect();

    let file = TaxonomyFile {
        version: 1,
        canonical_genres,
        ignored: Vec::new(),
    };

    let yaml = serde_yaml::to_string(&file)?;
    if let Some(parent) = path.parent() {
        std::fs::create_dir_all(parent)?;
    }
    std::fs::write(path, yaml)?;
    Ok(())
}
