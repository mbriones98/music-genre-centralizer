# Runbook

## Prerequisites

- **Rust toolchain** (1.85+ required for edition 2024, 1.95+ recommended). Install via [rustup](https://rustup.rs/):
  ```bash
  curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh
  ```
- **No other dependencies** for M2. Future milestones will add Node.js (for the Tauri/React UI).

## Building

```bash
# Debug build (fast compile, slow runtime — use during development)
cargo build

# Release build (slow compile, optimized runtime — use for scanning large libraries)
cargo build --release
```

The CLI binary lands at:
- Debug: `target/debug/gc-cli`
- Release: `target/release/gc-cli`

## Running

### Scan a directory (dry-run)

Point it at a folder containing MP3 files. Nothing is modified — it only reads tags and prints proposed mappings.

```bash
cargo run -p gc-cli -- scan /path/to/your/music

# Or run the binary directly after building
./target/debug/gc-cli scan /path/to/your/music
```

Example output:

```
Scanning: /Users/you/Music/albums
  track01.mp3                                        | Cloud Rap            | -> Hip Hop
  track02.mp3                                        | Rock                 | OK (already canonical)
  track03.mp3                                        | Vaporwave            | -> Electronic
  track04.mp3                                        | —                    | SKIP (empty genre)

Summary: 4 MP3s found, 3 matched, 1 unknown/skipped
```

**Reading the output:**
- `-> Hip Hop` — the genre will be remapped to the canonical genre shown
- `OK (already canonical)` — no change needed
- `UNKNOWN (skipped)` — the genre doesn't match any alias in the taxonomy
- `SKIP (empty genre)` — the file has no genre tag set
- `SKIP (no tags)` — the file has no metadata tags at all

### Scan and apply changes

```bash
cargo run -p gc-cli -- scan --apply /path/to/your/music
```

This writes the canonical genre to each MP3's TCON tag and preserves the original sub-genre in a TXXX:ORIGINAL_GENRE field.

**Unknown genres:** After the scan, any unmatched genres are presented interactively. For each unknown, you can:
- Type a number to accept a fuzzy-match suggestion
- Type any canonical genre name to map it
- Press Enter to skip

Accepted mappings are saved to your user taxonomy at:
- macOS: `~/Library/Application Support/genre-centralizer/taxonomy.yaml`
- Linux: `~/.config/genre-centralizer/taxonomy.yaml`

These persist across runs so the taxonomy self-heals over time.

**Warning:** `--apply` modifies files in place. To test safely, use a copy of your music or the test fixtures:

```bash
cp -r testdata /tmp/gc-test
cargo run -p gc-cli -- scan --apply /tmp/gc-test
```

To restore test fixtures after applying: `git checkout testdata/`

### Validate the taxonomy

```bash
cargo run -p gc-cli -- taxonomy validate
```

Checks the bundled taxonomy for ambiguous aliases (same normalized string mapping to different canonical genres).

### Scan the test fixtures

```bash
cargo run -p gc-cli -- scan testdata/
```

This uses the bundled synthetic MP3 files to verify the tool works without needing your own music library.

## Testing

```bash
# Run all tests (25 tests across taxonomy, classify, normalize, tagio)
cargo test

# Run only gc-core tests
cargo test -p gc-core

# Run tests for a specific module
cargo test -p gc-core normalize     # normalization tests
cargo test -p gc-core taxonomy      # taxonomy loading, fuzzy match, alias tests
cargo test -p gc-core classify      # genre classification tests
cargo test -p gc-core tagio         # MP3 tag write round-trip tests

# Run a single test by name
cargo test -p gc-core suggest_matches

# Run tests with output printed (even passing tests)
cargo test -- --nocapture
```

## Troubleshooting

### `cargo` not found

Source the Rust environment in your current shell:
```bash
source "$HOME/.cargo/env"
```

### Scan shows 0 MP3s found

- The tool currently only scans the immediate directory, not subdirectories (recursive scan comes in M3)
- Check that the files are actually `.mp3` — the extension check is case-insensitive but must be present
- Verify the directory path is correct: `ls /path/to/your/music/*.mp3`

### All genres show as UNKNOWN

The YAML taxonomy covers ~248 aliases across 16 canonical genres. If your library uses niche genres not in the list, they'll show as UNKNOWN. Use `--apply` to interactively resolve them and they'll be saved for future runs.

## Future commands (not yet implemented)

These will be added in later milestones:

```bash
# M3: Support for FLAC, M4A, OGG, WAV
cargo run -p gc-cli -- scan --apply /path/to/mixed-format-library

# M4: Launch the desktop app
cargo tauri dev
```
