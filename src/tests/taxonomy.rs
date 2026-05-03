use std::collections::HashMap;

use crate::{build_taxonomy, normalize};

#[test]
fn has_all_canonicals() {
    let tax = build_taxonomy();
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
    ];
    for name in canonicals {
        assert!(
            tax.contains_key(&normalize(name)),
            "Missing canonical: {name}"
        );
    }
}

#[test]
fn canonical_maps_to_self() {
    let tax = build_taxonomy();
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
    ];
    for name in canonicals {
        assert_eq!(
            tax.get(&normalize(name)).unwrap(),
            name,
            "Canonical {name} should map to itself"
        );
    }
}

#[test]
fn no_duplicate_normalized_keys() {
    let mappings: &[(&str, &str)] = &[
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
        ("Rap", "Hip Hop"),
        ("Cloud Rap", "Hip Hop"),
        ("Trap", "Hip Hop"),
        ("Drill", "Hip Hop"),
        ("Boom Bap", "Hip Hop"),
        ("Shoegaze", "Rock"),
        ("Indie Rock", "Rock"),
        ("Post-Rock", "Rock"),
        ("Alternative Rock", "Rock"),
        ("Punk", "Rock"),
        ("Grunge", "Rock"),
        ("House", "Electronic"),
        ("Tech House", "Electronic"),
        ("Techno", "Electronic"),
        ("Drum and Bass", "Electronic"),
        ("Dubstep", "Electronic"),
        ("Ambient", "Electronic"),
        ("IDM", "Electronic"),
        ("Trance", "Electronic"),
        ("Death Metal", "Metal"),
        ("Black Metal", "Metal"),
        ("Thrash Metal", "Metal"),
        ("Doom Metal", "Metal"),
    ];

    let mut seen = HashMap::new();
    for (alias, canonical) in mappings {
        let key = normalize(alias);
        if let Some(prev) = seen.insert(key.clone(), (*alias, *canonical)) {
            panic!(
                "Normalized key '{key}' is used by both '{}' -> '{}' and '{alias}' -> '{canonical}'",
                prev.0, prev.1
            );
        }
    }
}
