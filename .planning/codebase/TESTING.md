# Testing Patterns

**Analysis Date:** 2026-02-23

## Test Framework

**Status:** No tests present

**Codebase scan:**
- No `*.test.rs`, `*.spec.rs`, or `tests/` directory found
- `Cargo.toml` contains no `[dev-dependencies]` section
- No test configuration files (`tests.toml`, etc.)

**Why this matters:**
- Async operations in `mise.rs` module (fetching tools, parsing JSON) are untested
- UI rendering logic in `src/ui/` has no coverage
- Action dispatching through complex state machine untested

## Build & Dependencies

**Test runner (if implemented):** Would use `cargo test` (standard Rust testing)

**Current test support:**
- Cargo configured for standard testing (no custom test runner)
- Can run tests with: `cargo test` (when tests are added)
- Can run with output: `cargo test -- --nocapture`

**Async runtime:**
- Project uses `tokio` runtime; tests would need `#[tokio::test]` macro

## Test File Organization

**Current structure (None):**
- No co-located tests (e.g., `module.rs` tests in same file with `#[cfg(test)]` mod)
- No separate `tests/` directory for integration tests

**Recommended approach if tests are added:**
- Unit tests: Co-located in modules with `#[cfg(test)] mod tests { #[test] fn ... }`
- Integration tests: In `tests/` directory for full app flows (if ever needed)

## Testable Components

**High-value test areas:**

**1. Model transformations (in `src/model.rs`):**
- `InstalledTool::from_map()` - Converts BTreeMap from JSON to flat vector
- `OutdatedTool::from_map()` - Similar transformation
- `EnvVar::from_map()` - Map to vector with defaults
- `MiseSetting::from_json()` - JSON Value to typed setting

**Example structure if tests added:**
```rust
#[cfg(test)]
mod tests {
    use super::*;
    use std::collections::BTreeMap;

    #[test]
    fn test_installed_tool_from_map() {
        let mut map = BTreeMap::new();
        let version = InstalledToolVersion {
            version: "1.0.0".to_string(),
            requested_version: None,
            install_path: Some("/path".to_string()),
            source: None,
            symlinked_to: None,
            installed: true,
            active: true,
        };
        map.insert("rust".to_string(), vec![version]);

        let tools = InstalledTool::from_map(map);
        assert_eq!(tools.len(), 1);
        assert_eq!(tools[0].name, "rust");
        assert_eq!(tools[0].version, "1.0.0");
    }
}
```

**2. Filtering logic (in `src/app.rs`):**
- `update_filtered_tools()` - Fuzzy matching + highlight caching
- `update_filtered_registry()` - Multi-field scoring (name, description, aliases)
- `update_filtered_outdated()` - Similar pattern
- Filter reset on empty query

**Example structure:**
```rust
#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_filter_reset_on_empty_query() {
        let mut app = App::new(/* action_tx */);
        app.tools = vec![
            InstalledTool { name: "rust".into(), ... },
            InstalledTool { name: "node".into(), ... },
        ];
        app.search_query = String::new();

        app.update_filtered_tools();

        // When query empty, all items shown in original order
        assert_eq!(app.filtered_tools, vec![0, 1]);
        assert_eq!(app.tools_hl, vec![vec![], vec![]]);
    }

    #[test]
    fn test_fuzzy_filtering() {
        let mut app = App::new(/* action_tx */);
        app.tools = vec![
            InstalledTool { name: "rust".into(), version: "1.70".into(), ... },
            InstalledTool { name: "node".into(), version: "18.0".into(), ... },
        ];
        app.search_query = "rs".to_string();

        app.update_filtered_tools();

        // "rust" matches "rs", "node" doesn't
        assert_eq!(app.filtered_tools.len(), 1);
        assert_eq!(app.filtered_tools[0], 0); // rust at index 0
    }
}
```

**3. Action handling (in `src/app.rs` `handle_action()`):**
- Tab navigation (NextTab, PrevTab)
- Selection movement with bounds checking
- Search mode enter/exit
- Data loaded actions update state correctly

**4. Event mapping (in `src/event.rs`):**
- Key code to Action conversion
- Modifier key handling (Ctrl+C for quit)
- Mouse scroll to navigation
- Unhandled keys return None

**Example:**
```rust
#[cfg(test)]
mod tests {
    use super::*;
    use crossterm::event::{KeyCode, KeyModifiers};

    #[test]
    fn test_map_key_quit() {
        let code = KeyCode::Char('c');
        let modifiers = KeyModifiers::CONTROL;

        let action = map_key(code, modifiers);

        assert_eq!(action, Some(Action::Quit));
    }

    #[test]
    fn test_map_key_navigation() {
        assert_eq!(map_key(KeyCode::Down, KeyModifiers::empty()), Some(Action::MoveDown));
        assert_eq!(map_key(KeyCode::Up, KeyModifiers::empty()), Some(Action::MoveUp));
    }
}
```

## Mocking Considerations

**What to mock:**

**1. `mise` CLI calls (in integration tests):**
- Use mock subprocess or inject fake JSON responses
- `mise::fetch_tools()` calls `Command::new("mise")`
- Could be abstracted to trait for testing: `trait MiseFetcher { async fn fetch_tools() -> Result<...> }`

**Current blocker:** No trait abstraction; direct `Command::new()` calls in `mise.rs`

**Recommended pattern if added:**
```rust
#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_parse_valid_tools() {
        // Mock JSON response from mise
        let json = r#"{"rust": [{"version": "1.70", "active": true, ...}]}"#;

        // Would need to extract parse logic to testable function
        let result: Result<BTreeMap<String, Vec<InstalledToolVersion>>, _> =
            serde_json::from_str(json);

        assert!(result.is_ok());
    }
}
```

**What NOT to mock:**
- Struct construction (models are simple data containers)
- Match statements (logic is straightforward)
- Style/theme functions (output is deterministic, no side effects)

## UI Rendering Tests

**Current blocker:** No UI testing framework integrated

**Renderers are in `src/ui/*.rs` - structure:**
- `render(f: &mut Frame, area: Rect, app: &App)` functions
- Can be tested by creating dummy `Frame` and `Rect` (from ratatui)
- Assertions would check widgets drawn, not visual output

**Example structure (if added):**
```rust
#[cfg(test)]
mod tests {
    use super::*;
    use ratatui::backend::TestBackend;
    use ratatui::Terminal;

    #[test]
    fn test_render_loading_state() {
        let mut terminal = Terminal::new(TestBackend::new(80, 24)).unwrap();
        let app = App::new(/* ... */);
        // Set tools_state to Loading

        let mut render_called = false;
        terminal.draw(|f| {
            render(f, f.size(), &app);
            render_called = true;
        }).unwrap();

        assert!(render_called);
        // Could assert buffer contents contain "Loading tools..."
    }
}
```

## Coverage

**Requirements:** None enforced

**Current coverage:** 0% (no tests)

**Critical paths for testing (if coverage goals added):**
1. Model transformations (high value, deterministic)
2. Filter logic (complex, multi-field matching)
3. Action dispatch (state machine correctness)
4. Async spawning (ensures actions channel properly)

## Test Patterns (Proposed)

**Async test pattern (using tokio):**
```rust
#[tokio::test]
async fn test_async_operation() {
    // Setup
    let (tx, _rx) = tokio::sync::mpsc::unbounded_channel();

    // Execute
    match mise::fetch_tools().await {
        Ok(tools) => { /* assert on tools */ }
        Err(e) => panic!("Unexpected error: {}", e),
    }
}
```

**State mutation pattern:**
```rust
#[test]
fn test_state_mutation() {
    let mut app = App::new(/* action_tx */);

    // Before
    assert_eq!(app.search_active, false);

    // Mutate
    app.handle_action(Action::EnterSearch);

    // After
    assert_eq!(app.search_active, true);
}
```

**Error testing pattern:**
```rust
#[test]
fn test_error_handling() {
    let (tx, _rx) = tokio::sync::mpsc::unbounded_channel();
    let mut app = App::new(tx);

    // Simulate operation failure
    app.handle_action(Action::OperationFailed("Tool not found".to_string()));

    // Assert error stored in status message
    assert!(app.status_message.is_some());
}
```

## Key Functions to Test (by Priority)

**Priority 1 (Logic):**
- `InstalledTool::from_map()` - Deserialize transformation
- `update_filtered_tools()`, `update_filtered_registry()`, etc. - Fuzzy matching correctness
- `highlight_cached()` - Index to highlighted text rendering
- `App::handle_action()` - State transitions

**Priority 2 (Events):**
- `map_key()` - Key to action mapping
- `map_event()` - Event dispatch

**Priority 3 (UI):**
- Renderer functions with LoadState checks (empty, loading, loaded paths)
- Layout computations

---

*Testing analysis: 2026-02-23*
