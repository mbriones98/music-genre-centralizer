use std::path::Path;

use lofty::file::TaggedFileExt;
use lofty::id3::v2::Id3v2Tag;
use lofty::tag::Accessor;

use crate::tagio::write_canonical_and_preserve;

fn copy_fixture_to_temp(fixture_name: &str) -> tempfile::NamedTempFile {
    let fixture_path = Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("../../testdata")
        .join(fixture_name);
    let temp = tempfile::Builder::new()
        .suffix(".mp3")
        .tempfile()
        .expect("Failed to create temp file");
    std::fs::copy(&fixture_path, temp.path()).expect("Failed to copy fixture");
    temp
}

#[test]
fn writes_canonical_genre_to_tcon() {
    let temp = copy_fixture_to_temp("cloud_rap_track.mp3");

    write_canonical_and_preserve(temp.path(), "Hip Hop", "Cloud Rap").unwrap();

    let tagged_file = lofty::read_from_path(temp.path()).unwrap();
    let tag = tagged_file.primary_tag().expect("Should have a tag after write");
    assert_eq!(tag.genre().as_deref(), Some("Hip Hop"));
}

#[test]
fn writes_original_genre_to_txxx() {
    let temp = copy_fixture_to_temp("cloud_rap_track.mp3");

    write_canonical_and_preserve(temp.path(), "Hip Hop", "Cloud Rap").unwrap();

    let tagged_file = lofty::read_from_path(temp.path()).unwrap();
    let generic_tag = tagged_file.primary_tag().expect("Should have a tag");
    let id3v2: Id3v2Tag = generic_tag.clone().into();
    let original = id3v2.get_user_text("ORIGINAL_GENRE");
    assert_eq!(original, Some("Cloud Rap"));
}

#[test]
fn already_canonical_preserves_original() {
    let temp = copy_fixture_to_temp("rock_track.mp3");

    write_canonical_and_preserve(temp.path(), "Rock", "Rock").unwrap();

    let tagged_file = lofty::read_from_path(temp.path()).unwrap();
    let tag = tagged_file.primary_tag().unwrap();
    assert_eq!(tag.genre().as_deref(), Some("Rock"));

    let id3v2: Id3v2Tag = tag.clone().into();
    assert_eq!(id3v2.get_user_text("ORIGINAL_GENRE"), Some("Rock"));
}

#[test]
fn unsupported_format_returns_error() {
    let temp = tempfile::NamedTempFile::with_suffix(".flac").unwrap();
    std::fs::write(temp.path(), b"not a real flac").unwrap();

    let result = write_canonical_and_preserve(temp.path(), "Rock", "Shoegaze");
    assert!(result.is_err());
}
