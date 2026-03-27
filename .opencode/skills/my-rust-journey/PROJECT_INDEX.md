# Project Index - my-rust-journey

Quick reference for AI tools working with this repository.

## Project Type

Rust desktop application (Windows) for medical device barcode generation.

## Quick Facts

- **Language:** Rust 2024 Edition
- **GUI:** Slint 1.15
- **Primary App:** AbbottBarcodeGeneration
- **Secondary App:** LotID-Codec
- **Examples:** 14 learning files

## Entry Points

| Binary                  | Main File                             | Purpose                   |
| ----------------------- | ------------------------------------- | ------------------------- |
| AbbottBarcodeGeneration | `src/AbbottBarcodeGeneration/main.rs` | Medical barcode generator |
| LotID-Codec             | `src/LotID-Codec/main.rs`             | Lot ID encoder/decoder    |

## Core Business Logic

### Abbott Barcodes (`src/AbbottBarcodeGeneration/abbott.rs`)

- Line 11-54: Data structures
- Line 60-73: Config loading
- Line 224-267: Content builders
- Line 319-371: Generation engine

### LotID Codec (`src/LotID-Codec/codec.rs`)

- Line 1-6: Encoding table (43 chars)
- Line 8-18: Encode function
- Line 20-28: Decode function

## Dependencies

```
slint = "1.15"          # GUI
zxing-cpp = "0.5.0"    # Barcode
image = "0.25"          # Image processing
serde = "1"             # Serialization
arboard = "3"           # Clipboard
rfd = "0.17"            # File dialogs
chrono = "0.4.44"       # Date/time
```

## Build Commands

```bash
cargo run --bin AbbottBarcodeGeneration
cargo run --bin LotID-Codec
cargo run --example <name>
cargo test
cargo clippy
```

## Key Files

- `Cargo.toml` - Project configuration
- `build.rs` - Slint build integration
- `ui/*.slint` - UI definitions
- `assets/abbott_projects.json` - Project config data
- `docs/` - Design documentation
