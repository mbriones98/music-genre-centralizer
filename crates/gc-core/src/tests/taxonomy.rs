use crate::taxonomy::{normalize, Classification};
use crate::Taxonomy;

#[test]
fn load_default_succeeds() {
    Taxonomy::load_default().expect("Default taxonomy should load without errors");
}

#[test]
fn has_all_canonicals() {
    let tax = Taxonomy::load_default().unwrap();
    let canonicals = [
        "Hip Hop",
        "Rock",
        "Electronic",
        "Jazz",
        "Classical",
        "R&B",
        "Pop",
        "Country",
        "Metal",
        "Folk",
        "Reggae",
        "Latin",
        "Blues",
        "Punk",
        "Soundtrack",
        "World",
    ];
    for name in canonicals {
        assert!(
            tax.lookup().contains_key(&normalize(name)),
            "Missing canonical: {name}"
        );
    }
}

#[test]
fn canonical_maps_to_self() {
    let tax = Taxonomy::load_default().unwrap();
    let canonicals = [
        "Hip Hop",
        "Rock",
        "Electronic",
        "Jazz",
        "Classical",
        "Pop",
        "Country",
        "Metal",
        "Folk",
        "Reggae",
        "Latin",
        "Blues",
        "Punk",
        "Soundtrack",
        "World",
    ];
    for name in canonicals {
        assert_eq!(
            tax.lookup().get(&normalize(name)).unwrap(),
            name,
            "Canonical {name} should map to itself"
        );
    }
}

#[test]
fn suggest_matches_returns_close_canonicals() {
    let tax = Taxonomy::load_default().unwrap();
    let suggestions = tax.suggest_matches("Jaz", 3);
    assert!(
        suggestions.iter().any(|(name, _)| name == "Jazz"),
        "Should suggest Jazz for 'Jaz', got: {:?}",
        suggestions
    );
}

#[test]
fn suggest_matches_empty_for_distant_genre() {
    let tax = Taxonomy::load_default().unwrap();
    let suggestions = tax.suggest_matches("xyzzyplugh", 3);
    assert!(suggestions.is_empty(), "Should have no suggestions for gibberish");
}

#[test]
fn add_alias_works() {
    let mut tax = Taxonomy::load_default().unwrap();
    assert_eq!(tax.classify("Reggaeton"), Classification::Unknown);
    tax.add_alias("Reggaeton", "Latin").unwrap();
    assert_eq!(
        tax.classify("Reggaeton"),
        Classification::Mapped {
            canonical: "Latin".to_string()
        }
    );
}

#[test]
fn add_alias_rejects_nonexistent_canonical() {
    let mut tax = Taxonomy::load_default().unwrap();
    assert!(tax.add_alias("Foo", "NotAGenre").is_err());
}

#[test]
fn ambiguous_alias_rejected() {
    let yaml = r#"
version: 1
canonical_genres:
  - name: "Rock"
    aliases: ["Punk"]
  - name: "Punk"
    aliases: []
"#;
    let result = Taxonomy::from_yaml(yaml);
    assert!(result.is_err(), "Should reject ambiguous alias");
}
