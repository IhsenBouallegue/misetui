# Architecture

**Analysis Date:** 2026-02-23

## Pattern Overview

**Overall:** Event-driven TUI with state machine, async task spawning, and multi-modal input routing.

**Key Characteristics:**
- Multi-tab tabbed interface (8 tabs) with focus switching
- Centralized state management (App struct) with async data loading
- Three input modes: normal, search, and version picker (each with remapped keybindings)
- Fuzzy filtering with precomputed highlight indices to avoid render-time performance cost
- MPSC channel-based async action dispatch for long-running operations
- Popup-based confirmation and progress dialogs
- Ratatui-based terminal rendering with one renderer per tab

## Layers

**Event Layer:**
- Purpose: Translate keyboard/mouse events into action enums
- Location: `src/event.rs`
- Contains: EventHandler struct, event mapping (key codes → Action enum)
- Depends on: crossterm, tokio
- Used by: main event loop in `src/main.rs`
- Details: Three background tokio tasks handle ticks (250ms), renders (16ms), and event stream reading; all send to shared MPSC channel

**Application State Layer:**
- Purpose: Hold all application state (data, filters, UI state, load states)
- Location: `src/app.rs`
- Contains: App struct with 8 data collections, filtered indices, highlight caches, selection indices, search state, popup state
- Depends on: Action enum, model data structures, fuzzy_matcher for filtering
- Used by: main loop and action handler
- Details: Single source of truth for all UI state; handles all actions and maintains consistency across all tabs

**Data Model Layer:**
- Purpose: Define structures for mise tool data and UI models
- Location: `src/model.rs`
- Contains: InstalledTool, RegistryEntry, OutdatedTool, MiseTask, EnvVar, MiseSetting, ConfigFile, PruneCandidate
- Depends on: serde for JSON deserialization
- Used by: App state, mise wrapper layer
- Details: Flat structures optimized for table rendering; conversion methods (from_map) to flatten JSON hierarchies

**Mise Integration Layer:**
- Purpose: Async wrappers around mise CLI commands
- Location: `src/mise.rs`
- Contains: Async functions (fetch_tools, fetch_registry, install_tool, etc.)
- Depends on: tokio::process::Command, model types
- Used by: App async spawns in action handlers
- Details: All functions run `mise` command-line tool via tokio, parse JSON output, convert to model types

**Action Routing Layer:**
- Purpose: Route events through mode-specific keybindings
- Location: `src/main.rs` (remap_* functions)
- Contains: Three remap functions for normal, search, and version picker modes
- Depends on: Action enum
- Used by: Event loop before calling app.handle_action()
- Details: Each mode transforms Action::SearchInput(char) differently; unbound characters become Action::None in normal mode

**UI Rendering Layer:**
- Purpose: Draw frame-by-frame based on current state
- Location: `src/ui/` (one module per tab + shared utilities)
- Contains: Tab renderers (tools.rs, registry.rs, outdated.rs, tasks.rs, environment.rs, settings.rs, config.rs, doctor.rs), layout, highlight, footer, header, popup, sidebar
- Depends on: ratatui, theme, App state, highlight utilities
- Used by: Main loop's terminal.draw()
- Details: render() in ui/mod.rs delegates to current tab's renderer; all renderers use highlight_cached() for search highlights

**Terminal Layer:**
- Purpose: Initialize/restore terminal capabilities (raw mode, alternate screen, mouse)
- Location: `src/tui.rs`
- Contains: Tui type alias, init(), restore() functions
- Depends on: crossterm
- Used by: main.rs for setup/teardown
- Details: Panic hook ensures restore() is called on crash

**Theme Layer:**
- Purpose: Centralized color and style definitions
- Location: `src/theme.rs`
- Contains: ANSI color constants, style functions for different UI elements
- Depends on: ratatui::style
- Used by: All UI renderers
- Details: Matte Candy palette with RGB colors

## Data Flow

**Application Startup:**

1. main() initializes terminal (tui::init)
2. Creates App with action_tx channel sender
3. Calls app.start_fetch() which spawns 8 async tasks for each data type
4. Enters main event loop with tokio::select!

**Normal User Action (e.g., Install Tool):**

1. Event layer detects key press → creates Action enum
2. Action is remapped based on current mode (normal/search/version picker)
3. app.handle_action(Action) processes the action
4. Handler may spawn new tokio task that calls mise function via action_tx
5. Async task completes → sends Action::<Result> back through channel
6. Next iteration of main loop receives completion action
7. app.handle_action(completion_action) updates App state and possibly calls start_fetch() to refresh
8. Next terminal.draw() renders updated state

**Search Filter Update:**

1. User presses `/` → Action::EnterSearch
2. app.handle_action sets search_active = true
3. User types chars → Action::SearchInput(c) variants
4. app.handle_action pushes to search_query and calls update_all_filters()
5. update_filtered_* methods run SkimMatcherV2 on primary field
6. Results precomputed: fuzzy_indices() computed once, stored in *_hl parallel arrays
7. Next render() calls highlight_cached(text, &app.tools_hl[i]) — no fuzzy at render time
8. Renderer uses cached indices to highlight matched chars

**Popup Flow (Version Picker):**

1. User presses 'i' on Registry tab → Action::InstallTool
2. app.handle_action spawns async task to fetch versions
3. Task returns Action::VersionsLoaded(versions) through channel
4. app.handle_action creates Popup::VersionPicker in app.popup
5. Popup mode activates: main loop routes chars to remap_version_picker_action
6. Chars become Action::PopupSearchInput, j/k become Up/Down
7. User presses Enter → Action::Confirm
8. app.handle_action matches on Popup::VersionPicker, spawns install task
9. Task completes → popup dismissed, start_fetch() refreshes all data

**State Management:**

- LoadState enum (Loading/Loaded) tracks which data has arrived
- Renderers check state and show spinners while loading
- filtered_* arrays parallel to main data arrays (indices into main)
- *_hl arrays parallel to filtered_* arrays (highlight indices per filtered item)
- selection indices (tools_selected, registry_selected, etc.) index into filtered_* arrays

## Key Abstractions

**Tab Enum:**
- Purpose: Represent the 8 content areas of the interface
- Examples: Tab::Tools, Tab::Registry, Tab::Outdated, Tab::Tasks, Tab::Environment, Tab::Settings, Tab::Config, Tab::Doctor
- Pattern: Cheap Copy enum with index() method, label() method for display, ALL constant array

**Action Enum:**
- Purpose: All possible events and state transitions
- Examples: Action::MoveUp, Action::SearchInput(char), Action::ToolsLoaded(Vec<InstalledTool>)
- Pattern: Sum type covering navigation, search, data load, operations, system events

**Popup Enum:**
- Purpose: Modal dialogs that intercept input
- Variants: VersionPicker (with internal search and filter state), Confirm (with action_on_confirm), Progress, ToolDetail, Help
- Pattern: Option<Popup> in App; when Some, input remaps to popup mode; renderers check app.popup

**Focus Enum:**
- Purpose: Track whether user is in sidebar or content area
- Used by: move_selection() to decide whether to navigate tabs or items in current tab

**LoadState Enum:**
- Purpose: Track async data loading status
- Used by: Renderers to show loading spinners vs. content

**ConfirmAction Enum:**
- Purpose: Encode what should happen when user confirms a Popup::Confirm
- Variants: Uninstall { tool, version }, Prune, TrustConfig { path }, RunTask { task }

## Entry Points

**Main Event Loop:**
- Location: `src/main.rs` lines 16–53
- Triggers: Binary startup
- Responsibilities: Initialize terminal/app, spawn event handler, tokio::select! between events and async actions, call terminal.draw() and app.handle_action()

**App::start_fetch():**
- Location: `src/app.rs` lines 253–309
- Triggers: Initial startup, Action::Refresh, operation completion
- Responsibilities: Spawn 8 async tasks to load all data types, send results back to action_tx

**App::handle_action():**
- Location: `src/app.rs` lines 311–972
- Triggers: Every keystroke or async completion
- Responsibilities: State mutations, popup creation, async task spawning, data filtering, sorting

**UI::render():**
- Location: `src/ui/mod.rs` lines 20–39
- Triggers: Every frame (16ms interval via Action::Render)
- Responsibilities: Delegate to tab-specific renderers, layout management, popup rendering

**App::update_filtered_*():**
- Location: `src/app.rs` lines 1091–1300 (multiple methods)
- Triggers: search_active change or search_query change
- Responsibilities: Run SkimMatcherV2 once to get filtered indices and highlight indices, store in *_hl

## Error Handling

**Strategy:** Result types propagated from async operations to App state

**Patterns:**
- mise CLI commands return Result<String, String> with error messages
- Async tasks catch errors and send Action::OperationFailed(error_string) back
- app.handle_action displays errors in status_message (bottom-left, yellow text, TTL of 20 ticks)
- UI continues to display old state while error is shown; error auto-clears after ~5 seconds

## Cross-Cutting Concerns

**Logging:** Not implemented. Errors flow through status_message UI.

**Validation:** Models validate JSON deserialization via serde; invalid JSON shows parse error in status message.

**Authentication:** Not applicable; mise handles its own auth via CLI.

**Performance Optimization:**
- Highlight indices precomputed at filter step (App::update_filtered_*), not at render
- highlight_cached() runs once per visible item per frame, no fuzzy matching in hot path
- Spinner animation reuses frame counter, no extra allocations

**State Consistency:**
- App is single source of truth
- All mutations flow through handle_action()
- Async results update App only via action_tx → handle_action()
