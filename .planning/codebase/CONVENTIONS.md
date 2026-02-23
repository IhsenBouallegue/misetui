# Coding Conventions

**Analysis Date:** 2026-02-23

## Naming Patterns

**Files:**
- Snake case: `app.rs`, `mise.rs`, `event.rs`, `model.rs`, `theme.rs`, `tui.rs`
- UI modules follow content: `tools.rs`, `registry.rs`, `tasks.rs`, `environment.rs`, `settings.rs`, `config.rs`, `doctor.rs`, `popup.rs`, `sidebar.rs`, `layout.rs`, `header.rs`, `footer.rs`, `highlight.rs`

**Functions:**
- Snake case for all functions: `remap_normal_action()`, `highlight_cached()`, `update_filtered_tools()`, `fetch_tools()`, `run_mise()`
- Private functions use `fn` (not `pub fn`): `map_event()`, `map_key()`, `centered_rect()`
- Public functions explicitly marked: `pub fn` for module exports, `pub async fn` for async operations

**Variables:**
- Snake case throughout: `search_query`, `filtered_tools`, `tools_hl`, `selected_index`, `tools_state`
- Constants use SCREAMING_SNAKE_CASE: `BG`, `FG`, `RED`, `GREEN`, `YELLOW`, `MUTED`, `SURFACE`, `HIGHLIGHT` (in `theme.rs`)
- Color values defined as module-level constants: `pub const BG: Color = Color::Rgb(6, 12, 16)`

**Types:**
- Pascal case for structs and enums: `App`, `Tab`, `Action`, `Popup`, `Focus`, `LoadState`, `InstalledTool`, `RegistryEntry`, `OutdatedTool`, `MiseTask`, `EnvVar`, `MiseSetting`
- Enum variants use Pascal case: `Tab::Tools`, `Action::MoveUp`, `LoadState::Loading`, `Popup::VersionPicker`

**Acronyms:**
- Preserved in full: `MiseTask`, `EnvVar`, `TabTab`, not abbreviated unnecessarily

## Code Style

**Formatting:**
- Standard Rust formatting (follows `rustfmt` defaults, no custom `.rustfmt.toml` present)
- 4-space indentation
- Opening braces on same line for all constructs: `fn foo() {`, `match x {`, `impl Foo {`

**Linting:**
- No explicit `.clippy.toml` or configuration present; follows Rust standard linting
- Uses `#[allow(dead_code)]` on serde-deserialized structs that may not use all fields: `#[allow(dead_code)]` in `model.rs` for `ToolSource`, `InstalledToolVersion`, `OutdatedEntry`, `EnvVarEntry`

**Attributes:**
- `#[derive(Debug, Clone)]` standard for state/model structs
- `#[derive(Debug, Clone, Deserialize)]` for JSON-deserialized types
- `#[serde(rename = "type")]` for field name mapping (e.g., `ToolSource.source_type`)
- `#[serde(default)]` for optional JSON fields: widespread in `model.rs`
- `#[tokio::main]` on main async entry point in `main.rs`

## Import Organization

**Order:**
1. Standard library imports: `use std::...`
2. External crate imports: `use tokio::...`, `use ratatui::...`, `use futures::...`
3. Internal crate imports: `use crate::...`

**Examples from codebase:**
```rust
// From main.rs
use action::Action;
use app::{App, Popup};
use color_eyre::Result;
use event::EventHandler;
use tokio::sync::mpsc;
```

```rust
// From app.rs
use crate::action::Action;
use crate::mise;
use crate::model::{ConfigFile, EnvVar, ...};
use fuzzy_matcher::skim::SkimMatcherV2;
use fuzzy_matcher::FuzzyMatcher;
use std::collections::HashMap;
use tokio::sync::mpsc;
```

**Path Aliases:**
- `crate::` used consistently for internal imports
- No shorthand aliases; full module paths (e.g., `tokio::sync::mpsc`, not `use tokio::sync as ts`)

## Error Handling

**Patterns:**
- Use `Result<T, String>` for fallible async operations: All `mise.rs` functions return `Result<Vec<...>, String>` or `Result<String, String>`
- Error messages are formatted with context: `format!("Parse error: {e}")`, `format!("Failed to run mise: {e}")`
- Errors are logged to action channel and surface as status messages: `Action::OperationFailed(e)` sent on error

**Match patterns for Results:**
```rust
// From app.rs error handling
match mise::install_tool(&tool_clone, &version).await {
    Ok(msg) => {
        let _ = tx.send(Action::OperationComplete(msg));
    }
    Err(e) => {
        let _ = tx.send(Action::OperationFailed(e));
    }
}
```

**Ignoring send errors:**
- Pattern `let _ = tx.send(...)` used consistently to ignore channel send failures (safe in single-sender context)

**Validation:**
- Input validation implicit in type system (e.g., `usize` for indices prevents negative values)
- JSON parsing delegated to `serde_json::from_str()` with error context wrapping

## Logging

**Framework:** `println!` and `eprintln!` NOT used; instead status messages flow through `Action::OperationComplete` / `Action::OperationFailed`

**Patterns:**
- No explicit logging framework (println/eprintln avoided in app)
- Status messages shown via `app.status_message: Option<(String, usize)>` (text + TTL in ticks)
- Errors captured in action system and displayed to user: misuse operations bubble error messages to UI

## Comments

**When to Comment:**
- Few inline comments; code is self-documenting via clear naming
- Comments appear on complex logic: fuzzy matching, index caching, mode-switching logic

**Doc comments (///):**
- Used on public types and public methods
- Found on model structs: `/// A single installed tool version from \`mise ls -J\`.`
- Found on key functions: `/// Render \`text\` as a \`Line\`, bolding+yellowing every character whose index appears in \`indices\`...`
- Found on key module functions: `/// In normal mode, map char inputs to their bound actions`

**Examples:**
```rust
/// Represents the source of a tool configuration.
#[allow(dead_code)]
#[derive(Debug, Clone, Deserialize)]
pub struct ToolSource {
    #[serde(rename = "type")]
    pub source_type: String,
    pub path: String,
}
```

```rust
/// Render `text` as a `Line`, bolding+yellowing every character whose index
/// appears in `indices` (precomputed by the filter step, never recomputed at
/// render time).  Falls back to plain `normal`-styled text when the slice is
/// empty.
pub fn highlight_cached(text: &str, indices: &[usize], normal: Style) -> Line<'static> {
```

## Function Design

**Size:**
- Most functions under 50 lines; large functions are filter methods (100-120 lines) with clear loop/match structure
- Large method: `handle_action()` in `app.rs` (~500 lines total) decomposed via match arms on action type

**Parameters:**
- Prefer owned types in public APIs: `Tool: String` not `&str` for serialized data
- Prefer references in internal helpers: `&self`, `&mut self` for methods
- Async functions take references where possible: `async fn fetch_tools()` takes no params; `async fn install_tool(tool: &str, version: &str)`

**Return Values:**
- Async functions return `Result<T, String>` consistently (no custom error types)
- Sync methods return primitives or owned types: `pub fn label(&self) -> &'static str` for static strings
- Filter methods return `Vec<usize>` for indices into original array (no allocation of filtered copies)

**Async/await:**
- All `mise` CLI calls wrapped in `tokio::spawn()` to avoid blocking render loop
- Operations dispatch actions through channel: `tx.send(Action::...)` from spawned tasks

## Module Design

**Exports:**
- `pub fn/struct/enum` explicitly marks public API
- Module modules in `src/ui/mod.rs` re-exports all tab renderers: `pub use tools::render as render_tools;` etc.
- `src/main.rs` imports directly: `use ui::render`

**Barrel Files:**
- `src/ui/mod.rs` acts as module hub, re-exporting all render functions and layout utilities
- Clean public interface: `render()`, `AppLayout` available at `ui::*` level

**Struct Fields:**
- Public fields on data structs for simplicity: `pub tab`, `pub search_query`, `pub filtered_tools`
- Method actions through `handle_action()` to avoid scattered mutations

## Action Dispatch Pattern

**Pattern observed:**
- `Action` enum in `src/action.rs` centralizes all event types (navigation, data, operations)
- Remapping functions (`remap_normal_action`, `remap_search_action`, `remap_version_picker_action`) in `main.rs` transform raw key events to domain actions
- `App::handle_action()` processes actions via large match statement, triggering async work when needed

**Benefits:** Single event type simplifies mode switching and enables time-travel debugging.

## Filtering and Highlight Caching

**Fuzzy matching:**
- Uses `fuzzy_matcher::skim::SkimMatcherV2` for search
- Filter methods compute both scored indices and highlight indices in one pass
- Highlight indices cached in parallel arrays: `tools_hl`, `registry_hl`, `outdated_hl`, `tasks_hl`, `env_hl`, `settings_hl`
- Render functions call `highlight_cached(text, &app.tools_hl[i], style)` with precomputed indices (zero fuzzy calls at render time)

**Sorting:**
- Scored results sorted descending by fuzzy score: `.sort_by(|a, b| b.0.cmp(&a.0))`
- Table sort column tracked in `app.sort_column` and `app.sort_ascending`

---

*Convention analysis: 2026-02-23*
