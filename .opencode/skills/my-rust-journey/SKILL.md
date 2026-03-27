---
name: my-rust-journey
description: Understand and work with the my-rust-journey Rust project - a medical barcode generation tool with learning examples
version: 1.0.0
---

# my-rust-journey Skill

## Overview

This skill provides understanding of the `my-rust-journey` Rust project - a learning repository that evolved into practical medical device barcode generation tools.

## Project Structure

```
my-rust-journey/
├── Cargo.toml                    # Project config (edition 2024)
├── src/
│   ├── lib.rs                    # Shared library (add, greet functions)
│   ├── AbbottBarcodeGeneration/  # Main: Abbott medical barcode generator
│   │   ├── main.rs               # Slint GUI entry point
│   │   ├── abbott.rs             # Barcode business logic
│   │   ├── barcode.rs            # Barcode image generation
│   │   └── config.rs             # Config persistence
│   └── LotID-Codec/              # Secondary: Lot ID encoder/decoder
│       ├── main.rs               # Slint GUI entry point
│       ├── codec.rs              # 43-char encoding/decoding
│       ├── callbacks.rs          # UI event handlers
│       └── imaging.rs            # Image utilities
├── ui/
│   ├── main.slint                # Main UI definition
│   ├── barcode.slint             # Barcode generator UI
│   └── lotid.slint               # LotID codec UI
├── examples/                     # 14 Rust syntax examples
│   ├── hello.rs, variables.rs, for.rs, while.rs, Loop.rs
│   ├── functions.rs, struct.rs, match.rs
│   ├── calculator.rs, fibonacci.rs, guess_number.rs
│   ├── input.rs, test.rs, zxing.rs
├── assets/                       # JSON config, images
├── docs/                         # Design documentation
└── build.rs                      # Slint build integration
```

## Key Components

### 1. AbbottBarcodeGeneration (Primary Application)

Medical barcode generator for Abbott diagnostics projects.

**Supported Projects:**
- **CTNI** - 2 barcodes per set (red long + yellow short)
- **CK-MB** - 2 barcodes per set (red long + yellow short)  
- **Myo** - 3 barcodes per set (red long + yellow short + green short)
- **BNP** - 3 barcodes per set (red long + yellow short + green short)

**Barcode Format Rules:**
- Long codes: Standard PDF417, 4.0×2.0 cm
- Short codes: Compact PDF417, 7.4×1.8 cm, columns=2, eclevel=6
- Content structure: `A{SN}{prefix}{control_no}{suffix}{expiry}{project_bits}{trailing}`

**Key Files:**
- `abbott.rs:11-54` - Data structures (AbbottReagent, AbbottProject, AbbottProjectsConfig)
- `abbott.rs:224-267` - Content builders (encode_expiry, build_short_content, build_long_content)
- `abbott.rs:319-371` - Main generation function (generate_abbott_barcodes)
- `config.rs` - Config persistence (JSON serialization)

**Example Barcode Content:**
```
Long: A01137G81307UD00070420266201010300001H0000162001AAAGOAABTZAAINQABPTEBCAUMDXUWW...
Short: A01137H81307UD00
```

### 2. LotID-Codec (Secondary Application)

Encodes/decodes Lot IDs using a 43-character table.

**Key Logic (`codec.rs`):**
- 43-char encoding table: `0-9, A-Z, -, +, /, $, ., %, space`
- XOR obfuscation with `0xE19A`
- Base-43 conversion for 3-character codes
- `encode(id, lot)` → 3 chars
- `decode(chars)` → (id, lot)

### 3. Examples Directory

14 standalone Rust examples for learning:
- Basic: hello, variables, input
- Control flow: for, while, Loop, match
- Functions/Structs: functions, struct
- Algorithms: calculator, fibonacci, guess_number
- Testing: test
- Integration: zxing (barcode library demo)

## Technology Stack

| Component | Technology |
|-----------|------------|
| Language | Rust 2024 Edition |
| GUI Framework | Slint 1.15 |
| Barcode | zxing-cpp 0.5.0 |
| Image Processing | image 0.25, png 0.18 |
| Serialization | serde, serde_json |
| Clipboard | arboard 3 |
| File Dialogs | rfd 0.17 |
| Date/Time | chrono 0.4.44 |

## Build & Run Commands

```bash
# Run examples
cargo run --example hello
cargo run --example zxing

# Run applications
cargo run --bin AbbottBarcodeGeneration
cargo run --bin LotID-Codec

# Build release
cargo build --bin AbbottBarcodeGeneration --release
cargo build --bin LotID-Codec --release

# Development
cargo check
cargo test
cargo clippy
cargo fmt
```

## Architecture Patterns

### Multi-Binary Workspace
- Single `Cargo.toml` with multiple `[[bin]]` entries
- Shared library code in `src/lib.rs`
- Each binary has its own module structure

### Slint UI Integration
- UI defined in `.slint` files
- `slint::include_modules!()` macro in main.rs
- Callback-based event handling
- `Arc<Mutex<T>>` for shared state

### Config Management
- JSON serialization with serde
- File-based persistence
- Default fallbacks

### Barcode Generation Flow
1. User input → Config struct
2. Config → Content string (build_long_content/build_short_content)
3. Content → Barcode image (zxing-cpp)
4. Image → Slint display + PNG export

## Common Tasks

### Adding New Abbott Project
1. Add project definition in `abbott.rs:default_abbott_projects()`
2. Define reagents with prefixes, project_bits, trailing data
3. Set control_no_suffix and default values

### Modifying Barcode Format
1. Edit `short_config()` or `long_config()` in `abbott.rs`
2. Adjust dimensions, columns, eclevel as needed

### Adding New Example
1. Create `examples/new_example.rs`
2. Run with `cargo run --example new_example`

### Debugging UI Issues
1. Check `.slint` files in `ui/` directory
2. Verify callback setup in `main.rs`
3. Check Slint documentation for widget properties

## Business Logic Reference

### Barcode Content Structure

**Long Barcode:**
```
A{5-digit SN}{prefix G}{5-digit control_no}{suffix}{8-digit expiry DDMMYYYY}{13-digit project_bits}{trailing_data}
```

**Short Barcode:**
```
A{5-digit SN}{prefix H/J}{5-digit control_no}{suffix}
```

### Prefix Codes
- `G` - Long barcode separator
- `H` - Yellow short barcode separator  
- `J` - Green short barcode separator

### Control No. Suffixes
- `UD00` - CTNI project
- `UN24` - CK-MB, Myo, BNP projects

## File Locations Reference

| Purpose | Path |
|---------|------|
| Main config | `Cargo.toml` |
| Abbott logic | `src/AbbottBarcodeGeneration/abbott.rs` |
| LotID logic | `src/LotID-Codec/codec.rs` |
| UI definitions | `ui/*.slint` |
| Project config | `assets/abbott_projects.json` |
| Design docs | `docs/软件设计.md` |
| Build script | `build.rs` |

## Notes

- Windows subsystem set to "windows" (no console)
- zxing-cpp compiled with opt-level 3 for performance
- Config auto-saves on generation
- Auth system for Abbott mode (username: relia, password: relia-abbott)
