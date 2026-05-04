use crate::taxonomy::Classification;
use crate::Taxonomy;

#[test]
fn empty_genre() {
    let tax = Taxonomy::load_default().unwrap();
    assert_eq!(tax.classify(""), Classification::Empty);
}

#[test]
fn already_canonical() {
    let tax = Taxonomy::load_default().unwrap();
    assert_eq!(tax.classify("Rock"), Classification::AlreadyCanonical);
    assert_eq!(tax.classify("Jazz"), Classification::AlreadyCanonical);
    assert_eq!(tax.classify("Pop"), Classification::AlreadyCanonical);
}

#[test]
fn known_alias() {
    let tax = Taxonomy::load_default().unwrap();
    assert_eq!(
        tax.classify("Cloud Rap"),
        Classification::Mapped {
            canonical: "Hip Hop".to_string()
        }
    );
    assert_eq!(
        tax.classify("Shoegaze"),
        Classification::Mapped {
            canonical: "Rock".to_string()
        }
    );
    assert_eq!(
        tax.classify("Techno"),
        Classification::Mapped {
            canonical: "Electronic".to_string()
        }
    );
}

#[test]
fn punctuation_and_case_variants() {
    let tax = Taxonomy::load_default().unwrap();
    let expected = Classification::Mapped {
        canonical: "Hip Hop".to_string(),
    };
    assert_eq!(tax.classify("Hip-Hop"), expected);
    assert_eq!(tax.classify("hip hop"), expected);
    assert_eq!(tax.classify("HIP HOP"), expected);
    assert_eq!(tax.classify("Hip_Hop"), expected);
}

#[test]
fn unknown_genre() {
    let tax = Taxonomy::load_default().unwrap();
    assert_eq!(tax.classify("Reggaeton"), Classification::Unknown);
}

#[test]
fn metal_subgenres() {
    let tax = Taxonomy::load_default().unwrap();
    let expected = Classification::Mapped {
        canonical: "Metal".to_string(),
    };
    assert_eq!(tax.classify("Death Metal"), expected);
    assert_eq!(tax.classify("death-metal"), expected);
    assert_eq!(tax.classify("DOOM METAL"), expected);
}
