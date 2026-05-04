use crate::taxonomy::normalize;

#[test]
fn lowercases() {
    assert_eq!(normalize("Rock"), "rock");
    assert_eq!(normalize("JAZZ"), "jazz");
}

#[test]
fn strips_spaces() {
    assert_eq!(normalize("Hip Hop"), "hiphop");
    assert_eq!(normalize("Indie Rock"), "indierock");
}

#[test]
fn strips_punctuation() {
    assert_eq!(normalize("Hip-Hop"), "hiphop");
    assert_eq!(normalize("R&B"), "rb");
    assert_eq!(normalize("Drum & Bass"), "drumbass");
    assert_eq!(normalize("Rock!"), "rock");
}

#[test]
fn strips_underscores() {
    assert_eq!(normalize("Hip_Hop"), "hiphop");
    assert_eq!(normalize("death_metal"), "deathmetal");
}

#[test]
fn mixed_separators_converge() {
    assert_eq!(normalize("Hip-Hop"), normalize("Hip Hop"));
    assert_eq!(normalize("Hip_Hop"), normalize("HipHop"));
    assert_eq!(normalize("hip hop"), normalize("HIP-HOP"));
}

#[test]
fn empty_string() {
    assert_eq!(normalize(""), "");
}

#[test]
fn already_clean() {
    assert_eq!(normalize("rock"), "rock");
    assert_eq!(normalize("pop"), "pop");
}
