# Technology Stack

**Analysis Date:** 2026-02-23

## Languages

**Primary:**
- Rust 2021 edition - Entire application codebase

## Runtime

**Environment:**
- Tokio 1.x - Async runtime for concurrent task execution

**Package Manager:**
- Cargo - Official Rust package manager
- Lockfile: `Cargo.lock` present

## Frameworks

**Core:**
- Ratatui 0.30 - Terminal user interface rendering and widget system
- Crossterm 0.28 - Terminal manipulation and event stream handling (with `event-stream` feature)

**Async:**
- Tokio 1 (full features) - Async runtime with all runtime features enabled (`features = ["full"]`)
- Futures 0.3 - Async primitives and combinators

**Error Handling & Panics:**
- Color-eyre 0.6 - Pretty error reporting and panic hooks

**Fuzzy Matching:**
- fuzzy-matcher 0.3 - Fuzzy string matching using SkimMatcher algorithm

**Serialization:**
- Serde 1 (with `derive` feature) - Serialization/deserialization framework
- Serde_json 1 - JSON encoding and decoding

**Text Rendering:**
- unicode-width 0.2 - Unicode character width calculation for terminal alignment

## Key Dependencies

**Critical:**
- `ratatui` 0.30 - Core rendering engine; entire UI system depends on this for drawing terminal widgets
- `tokio` 1 (full) - Async runtime; all async operations for mise CLI calls depend on this
- `fuzzy-matcher` 0.3 - Enables search functionality; impacts performance of all filtering operations (`SkimMatcherV2` in `src/app.rs`)
- `crossterm` 0.28 - Terminal event handling; required for keyboard/mouse input and alternate screen mode

**Infrastructure:**
- `color-eyre` 0.6 - Provides pretty error formatting and stack traces with context
- `serde` + `serde_json` - Parses JSON output from `mise` CLI (all data fetching in `src/mise.rs` depends on this)

## Configuration

**Environment:**
- No explicit environment variables required at runtime
- `.env` file not used
- Configuration comes entirely from `mise` CLI tool (external dependency)

**Build:**
- `Cargo.toml` - Single manifest file
- `Cargo.lock` - Exact dependency versions locked
- Edition 2021 Rust syntax

## Platform Requirements

**Development:**
- Rust toolchain (1.56+, supports 2021 edition)
- Cargo installed
- `mise` CLI tool must be available in `PATH` for development

**Production:**
- Linux, macOS, Windows (any platform supporting Rust and Tokio)
- `mise` CLI tool must be installed and available in `PATH` at runtime
- Terminal with 24-bit color support recommended (uses Ratatui's default colors)

## External Dependencies

**CLI Tool Integration:**
- `mise` (jdx.dev) - Version manager for dev tools; all core functionality depends on calling this external binary

---

*Stack analysis: 2026-02-23*
