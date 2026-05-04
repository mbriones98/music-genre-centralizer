use std::path::Path;

use anyhow::{Context, Result, bail};
use lofty::config::WriteOptions;
use lofty::file::{FileType, TaggedFileExt};
use lofty::id3::v2::Id3v2Tag;
use lofty::tag::{Accessor, Tag, TagExt, TagType};

pub fn write_canonical_and_preserve(path: &Path, canonical: &str, original: &str) -> Result<()> {
    let tagged_file = lofty::read_from_path(path)
        .with_context(|| format!("Failed to read: {}", path.display()))?;

    match tagged_file.file_type() {
        FileType::Mpeg => write_mp3(path, &tagged_file, canonical, original),
        other => bail!("Unsupported format {other:?} — only MP3 is supported in this version"),
    }
}

fn write_mp3(
    path: &Path,
    tagged_file: &lofty::file::TaggedFile,
    canonical: &str,
    original: &str,
) -> Result<()> {
    let generic_tag = tagged_file
        .primary_tag()
        .cloned()
        .unwrap_or_else(|| Tag::new(TagType::Id3v2));

    let mut id3v2: Id3v2Tag = generic_tag.into();

    id3v2.set_genre(canonical.to_string());
    id3v2.insert_user_text(String::from("ORIGINAL_GENRE"), original.to_string());

    id3v2
        .save_to_path(path, WriteOptions::new())
        .with_context(|| format!("Failed to write tags: {}", path.display()))?;

    Ok(())
}
