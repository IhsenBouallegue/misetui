# Codebase Structure

**Analysis Date:** 2026-02-23

## Directory Layout

```
misetui/
├── src/
│   ├── main.rs              # Event loop, mode-specific input remapping
│   ├── app.rs               # App state, action handler, filter/sort logic
│   ├── action.rs            # Action enum (all events)
│   ├── event.rs             # EventHandler, crossterm integration
│   ├── mise.rs              # Async wrappers for mise CLI commands
│   ├── model.rs             # Data structures (InstalledTool, RegistryEntry, etc.)
│   ├── tui.rs               # Terminal init/restore
│   ├── theme.rs             # Color constants and style definitions
│   └── ui/
│       ├── mod.rs           # render() entry point, tab delegation
│       ├── layout.rs        # AppLayout for 3-pane structure
│       ├── highlight.rs     # highlight_cached() utility
│       ├── header.rs        # Top banner with outdated count
│       ├── sidebar.rs       # Left panel with 8 tabs
│       ├── footer.rs        # Bottom banner with help/status
│       ├── popup.rs         # Modal dialogs (version picker, confirm, progress, detail, help)
│       ├── tools.rs         # Tools tab renderer (table of installed tools)
│       ├── registry.rs      # Registry tab renderer (available tools)
│       ├── outdated.rs      # Outdated tab renderer (upgradeable tools)
│       ├── tasks.rs         # Tasks tab renderer (mise tasks)
│       ├── environment.rs   # Environment tab renderer (env vars)
│       ├── settings.rs      # Settings tab renderer (mise settings)
│       ├── config.rs        # Config tab renderer (config files)
│       └── doctor.rs        # Doctor tab renderer (mise doctor output)
├── Cargo.toml               # Package metadata and dependencies
├── Cargo.lock               # Locked dependency versions
├── README.md                # User documentation
├── LICENSE                  # MIT license
└── target/                  # Build artifacts (not committed)
```

## Directory Purposes

**`src/`:**
- Purpose: All Rust source code
- Contains: Core logic, UI renderers, data models, event handling
- Key files: main.rs (entry point), app.rs (state), ui/mod.rs (rendering)

**`src/ui/`:**
- Purpose: Terminal UI rendering modules
- Contains: One renderer per tab, shared utilities (layout, highlight, popup)
- Key files: mod.rs (render entry point), popup.rs (all dialogs), layout.rs (3-pane layout)
- Pattern: Each tab renderer has signature `pub fn render(f: &mut Frame, area: Rect, app: &App)`

**`target/`:**
- Purpose: Cargo build output
- Generated: Yes
- Committed: No (in .gitignore)

## Key File Locations

**Entry Points:**
- `src/main.rs`: Binary entry point (main function), event loop, mode-specific key remapping
- `src/ui/mod.rs`: render() function dispatches to tab renderers

**Application State:**
- `src/app.rs`: App struct (all UI state, data, filters), handle_action() method

**Data Models:**
- `src/model.rs`: InstalledTool, RegistryEntry, OutdatedTool, MiseTask, EnvVar, MiseSetting, ConfigFile, PruneCandidate

**Async Integration:**
- `src/mise.rs`: fetch_tools, fetch_registry, fetch_versions, install_tool, uninstall_tool, update_tool, run_task, etc.
- `src/event.rs`: EventHandler spawning 3 background tokio tasks for events, ticks, renders

**Theme and Styling:**
- `src/theme.rs`: Color constants, style() functions for each UI element
- `src/ui/highlight.rs`: highlight_cached() for search result highlighting

**Popup/Modal Logic:**
- `src/ui/popup.rs`: All popup rendering; version picker, confirm, progress, tool detail, help

**Tab Renderers:**
- `src/ui/tools.rs`: Installed tools table (name, version, active, source)
- `src/ui/registry.rs`: Available tools registry (short name, description, aliases)
- `src/ui/outdated.rs`: Outdated tools (name, current, latest, requested)
- `src/ui/tasks.rs`: Mise tasks (name, description, source)
- `src/ui/environment.rs`: Environment variables (name, value, source, tool)
- `src/ui/settings.rs`: Mise settings (key, value, value_type)
- `src/ui/config.rs`: Config files (path, tools list)
- `src/ui/doctor.rs`: Mise doctor output (lines of text)

## Naming Conventions

**Files:**
- `snake_case.rs` for all Rust files
- Tab renderers named after tab: `tools.rs`, `registry.rs`, `outdated.rs`, etc.
- Utilities prefixed by function: `layout.rs`, `highlight.rs`, `popup.rs`, `header.rs`, `footer.rs`, `sidebar.rs`

**Functions:**
- `pub fn render(f: &mut Frame, area: Rect, app: &App)` — standard renderer signature
- `pub async fn fetch_*()` — async data loaders in mise.rs
- `pub fn *_filter_*()` — filtering/search methods in app.rs
- `handle_action()` — single action dispatcher
- `remap_*_action()` — mode-specific input mappers in main.rs

**Variables:**
- `app` — the App state struct
- `tools`, `registry`, `outdated`, `tasks`, `env_vars`, `settings`, `configs`, `doctor_lines` — data collections
- `filtered_*` — indices into full data collections (filtered by search or other criteria)
- `*_hl` — highlight indices parallel to `filtered_*` arrays
- `*_selected` — current selection index into `filtered_*` (per tab)
- `*_state` — LoadState::Loading or LoadState::Loaded for each data type

**Constants:**
- `Tab::Tools`, `Tab::Outdated`, etc. — enum variants (PascalCase)
- `SPINNER` — character array (UPPERCASE for constants)
- `BG`, `FG`, `RED`, `GREEN`, `YELLOW`, `MUTED`, `SURFACE`, `HIGHLIGHT` — color constants

**Types:**
- Action enum: variants are PascalCase, tuple variants carry data (SearchInput(char), MoveDown)
- Popup enum: VersionPicker, Confirm, Progress, ToolDetail, Help
- LoadState enum: Loading, Loaded
- Tab enum: Tools, Outdated, Registry, Tasks, Environment, Settings, Config, Doctor
- Model structs: InstalledTool, RegistryEntry, OutdatedTool, MiseTask, EnvVar, MiseSetting, ConfigFile

## Where to Add New Code

**New Feature (e.g., new tab):**
1. Add Tab variant to Tab enum in `src/app.rs` (line 13–22)
2. Add Tab::ALL entry and label()
3. Add data collection to App struct: `pub new_data: Vec<NewType>`
4. Add load state: `pub new_state: LoadState`
5. Add selection index: `pub new_selected: usize`
6. Add filter arrays: `pub filtered_new: Vec<usize>`, `pub new_hl: Vec<Vec<usize>>`
7. Create `src/ui/new_tab.rs` with `pub fn render(f: &mut Frame, area: Rect, app: &App)`
8. Add to ui/mod.rs render match: `Tab::NewTab => new_tab::render(f, layout.content, app)`
9. Add fetch function in `src/mise.rs`: `pub async fn fetch_new_data() -> Result<Vec<NewType>, String>`
10. Call from app.start_fetch() and add action handler
11. Add filter update in app.rs: `fn update_filtered_new(&mut self)`

**New Popup Type (e.g., new modal dialog):**
1. Add variant to Popup enum in `src/app.rs` (line 70–92)
2. Add handling in app.rs action handler (e.g., line 480 for InstallTool)
3. Create render function in `src/ui/popup.rs` and call from render() match
4. Handle async result or confirmation in handle_action Confirm branch

**New Action:**
1. Add variant to Action enum in `src/action.rs`
2. Add event mapping in `src/event.rs` if it's a key
3. Add handling in app.rs handle_action() method
4. Add mode-specific remapping if needed in main.rs remap_* functions

**New Helper/Utility:**
- Standalone functions: Add to utils file or appropriate module
- Styling function: Add to `src/theme.rs` (return Style)
- Rendering helper: Add to relevant ui file or create new `ui/utilities.rs`

## Special Directories

**`target/`:**
- Purpose: Cargo build artifacts
- Generated: Yes (via `cargo build`)
- Committed: No (ignored via .gitignore)

**`.planning/`:**
- Purpose: GSD planning documents
- Generated: No (manually created)
- Committed: Yes (planning docs)

## Import Patterns

**Standard imports in renderers:**
```rust
use super::highlight::highlight_cached;  // or use crate::ui::highlight
use crate::app::{App, LoadState};
use crate::theme;
use ratatui::layout::{Constraint, Direction, Layout, Rect};
use ratatui::style::Style;
use ratatui::text::{Line, Span};
use ratatui::widgets::{Block, Borders, Cell, Paragraph, Row, Table};
use ratatui::Frame;
```

**Standard imports in app.rs:**
```rust
use crate::action::Action;
use crate::mise;
use crate::model::{...};
use fuzzy_matcher::skim::SkimMatcherV2;
use fuzzy_matcher::FuzzyMatcher;
use std::collections::HashMap;
use tokio::sync::mpsc;
```

**Standard imports in main.rs:**
```rust
use action::Action;
use app::{App, Popup};
use color_eyre::Result;
use event::EventHandler;
use tokio::sync::mpsc;
```

## Configuration

**Theme/Colors:**
- All colors defined in `src/theme.rs` as constants
- To change palette: modify RGB values or Color::Ansi variants in theme.rs
- Each UI element has a dedicated style function (title(), table_header(), etc.)

**Key Bindings:**
- Normal mode: `src/main.rs` remap_normal_action() lines 64–88
- Search mode: `src/main.rs` remap_search_action() lines 91–104
- Version picker mode: `src/main.rs` remap_version_picker_action() lines 107–120

**UI Layout:**
- Fixed 3-pane: header (3 rows) | body (sidebar 16 cols + content min 20) | footer (2 rows)
- Defined in `src/ui/layout.rs`
- Adjustable: Constraint::Length(N) for fixed sizes, Constraint::Min(N) for min sizes

**Performance Tuning:**
- Tick interval: 250ms (src/event.rs line 17)
- Render interval: 16ms (~60 FPS) (src/event.rs line 28)
- Status message TTL: 20 ticks ≈ 5 seconds (src/app.rs line 405)

## Testing

Tests location: Not found (no tests in codebase)

Current state: No test files exist (no *.test.rs or *.spec.rs files)

To add tests:
1. Create `src/app.rs` → `src/app/mod.rs` (refactor)
2. Create `src/app/tests.rs` or `tests/integration_tests.rs`
3. Mock mise CLI via a test helper layer
4. Test filter/sort logic, action handlers, state transitions
