# Runbook

## Prerequisites

- **Rust toolchain** (1.75+ required, 1.95+ recommended). Install via [rustup](https://rustup.rs/):
  ```bash
  curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh
  ```
- **No other dependencies** for M1. Future milestones will add Node.js (for the Tauri/React UI).

## Building

```bash
# Debug build (fast compile, slow runtime — use during development)
cargo build

# Release build (slow compile, optimized runtime — use for scanning large libraries)
cargo build --release
```

The binary lands at:
- Debug: `target/debug/genre-centralizer`
- Release: `target/release/genre-centralizer`

## Running

### Scan a directory (dry-run)

Point it at a folder containing MP3 files. Nothing is modified — it only reads tags and prints proposed mappings.

```bash
cargo run -- /path/to/your/music

# Or run the binary directly after building
./target/debug/genre-centralizer /path/to/your/music
```

Example output:

```
Scanning: /Users/you/Music/albums
  track01.mp3                                        | Cloud Rap            | -> Hip Hop
  track02.mp3                                        | Rock                 | OK (already canonical)
  track03.mp3                                        | Vaporwave            | UNKNOWN
  track04.mp3                                        | —                    | SKIP (empty genre)

Summary: 4 MP3s found, 2 matched, 2 unknown/skipped
```

**Reading the output:**
- `-> Hip Hop` — the genre will be remapped to the canonical genre shown
- `OK (already canonical)` — no change needed
- `UNKNOWN` — the genre doesn't match any alias in the taxonomy
- `SKIP (empty genre)` — the file has no genre tag set
- `SKIP (no tags)` — the file has no metadata tags at all

### Scan the test fixtures

```bash
cargo run -- testdata/
```

This uses the bundled synthetic MP3 files to verify the tool works without needing your own music library.

## Testing

```bash
# Run all tests
cargo test

# Run tests for a specific module
cargo test normalize       # just normalization tests
cargo test taxonomy        # just taxonomy tests
cargo test classify        # just classification tests

# Run a single test by name
cargo test mixed_separators

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

The hardcoded taxonomy in M1 only covers ~30 aliases across 10 canonical genres (Hip Hop, Rock, Electronic, Jazz, Classical, R&B, Pop, Country, Metal, Folk). If your library uses genres not in this list, they'll show as UNKNOWN. M2 will replace this with a comprehensive YAML taxonomy.

## Future commands (not yet implemented)

These will be added in later milestones:

```bash
# M2: Apply genre changes to MP3 files
cargo run -- scan --apply /path/to/music

# M2: Validate the taxonomy file for ambiguities
cargo run -- taxonomy validate

# M4: Launch the desktop app
cargo tauri dev
```
