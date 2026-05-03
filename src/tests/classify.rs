use crate::{build_taxonomy, classify, Classification};

#[test]
fn empty_genre() {
    let tax = build_taxonomy();
    assert_eq!(classify("", &tax), Classification::Empty);
}

#[test]
fn already_canonical() {
    let tax = build_taxonomy();
    assert_eq!(classify("Rock", &tax), Classification::AlreadyCanonical);
    assert_eq!(classify("Jazz", &tax), Classification::AlreadyCanonical);
    assert_eq!(classify("Pop", &tax), Classification::AlreadyCanonical);
}

#[test]
fn known_alias() {
    let tax = build_taxonomy();
    assert_eq!(
        classify("Cloud Rap", &tax),
        Classification::Mapped {
            canonical: "Hip Hop".to_string()
        }
    );
    assert_eq!(
        classify("Shoegaze", &tax),
        Classification::Mapped {
            canonical: "Rock".to_string()
        }
    );
    assert_eq!(
        classify("Techno", &tax),
        Classification::Mapped {
            canonical: "Electronic".to_string()
        }
    );
}

#[test]
fn punctuation_and_case_variants() {
    let tax = build_taxonomy();
    let expected = Classification::Mapped {
        canonical: "Hip Hop".to_string(),
    };
    assert_eq!(classify("Hip-Hop", &tax), expected);
    assert_eq!(classify("hip hop", &tax), expected);
    assert_eq!(classify("HIP HOP", &tax), expected);
    assert_eq!(classify("Hip_Hop", &tax), expected);
}

#[test]
fn unknown_genre() {
    let tax = build_taxonomy();
    assert_eq!(classify("Vaporwave", &tax), Classification::Unknown);
    assert_eq!(classify("Reggaeton", &tax), Classification::Unknown);
}

#[test]
fn metal_subgenres() {
    let tax = build_taxonomy();
    let expected = Classification::Mapped {
        canonical: "Metal".to_string(),
    };
    assert_eq!(classify("Death Metal", &tax), expected);
    assert_eq!(classify("death-metal", &tax), expected);
    assert_eq!(classify("DOOM METAL", &tax), expected);
}
