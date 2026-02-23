# Codebase Concerns

**Analysis Date:** 2026-02-23

## Complexity & Maintainability Issues

**App State Management - Monolithic State Structure:**
- Issue: `src/app.rs` is 1501 lines with 35 functions, combining state management, filtering, action handling, and sorting logic all in one module.
- Files: `src/app.rs` (primary concern)
- Impact: Makes the `App` struct harder to test and reason about. Adding new tabs or features requires modifying this large file.
- Fix approach: Extract filtering/search into separate module (`src/filter.rs`), consider splitting action handling into behavior modules, break sort logic into dedicated type.

**Extensive Clone Usage:**
- Issue: 48 `.clone()` calls in `src/app.rs` alone, particularly in search filter methods (`update_filtered_*` functions) and action handlers. Occurs heavily in `tokio::spawn` closures for tool operations.
- Files: `src/app.rs` (lines 1099-1300 filter methods especially), `src/mise.rs` (lines 64, 70, etc. in command builders)
- Impact: Performance degradation during search on large datasets, unnecessary memory allocations during async operations.
- Fix approach: Use `Arc<String>` for frequently cloned data in filters, refactor async spawns to minimize string duplication, consider `Cow<str>` in data models.

**App::handle_action - God Function:**
- Issue: Single `handle_action` method (lines 311-972 in app.rs) contains all event handling and business logic with deeply nested matches and control flow.
- Files: `src/app.rs` lines 311-972
- Impact: Difficult to locate where specific actions are handled, complex to test individual action outcomes, high cognitive load for modifications.
- Fix approach: Split into per-tab handlers (`handle_tools_action`, `handle_registry_action`), extract popup logic into separate method, create action handler trait for operations.

**Filter State Duplication:**
- Issue: Parallel `filtered_*` arrays (tools, registry, configs, etc.) and corresponding `*_hl` highlight cache arrays maintained separately. This creates 16 state variables for filtering in App struct.
- Files: `src/app.rs` lines 150-165
- Impact: Risk of cache coherency bugs where `filtered_tools` and `tools_hl` get out of sync, error-prone when adding new data types.
- Fix approach: Create generic `FilteredList<T>` struct that maintains indices and highlights together, use it for all filtered data.

## Performance & Scaling Issues

**Search Filter Performance on Large Datasets:**
- Issue: `update_all_filters()` (line 1067) runs fuzzy matching on all 8 tabs even if only one tab is visible. Each filter clones the search query and creates new `SkimMatcherV2` instances.
- Files: `src/app.rs` lines 1067-1076, 1091-1300
- Impact: Noticeably slow on systems with hundreds of tools/tasks/env vars in registry. Every keystroke triggers all filter computations.
- Fix approach: Add tab-aware filtering (only compute visible tab's filter), memoize matcher instances, consider debouncing filter updates, potentially use SIMD fuzzy matcher.

**Async Task Proliferation Without Limits:**
- Issue: Every operation spawns unbounded `tokio::spawn` tasks without rate limiting or cancellation support. Fetching 8 data sources simultaneously at startup and after each operation.
- Files: `src/app.rs` lines 253-309 (start_fetch), `src/app.rs` lines 631-912 (async operations in handle_action)
- Impact: On slow/unreliable networks, multiple overlapping refreshes could pile up. No way to cancel in-flight operations or prevent race conditions between operations.
- Fix approach: Add task handles to App struct, implement cancellation tokens, serialize critical operations or limit concurrent count, add timeout handling.

**N+1 Query Pattern for Outdated Tools:**
- Issue: Tools displayed with outdated indicator by looking up in HashMap (line 99-113 in tools.rs), but outdated map rebuild happens after every refresh without deduplication.
- Files: `src/app.rs` lines 377-384 (OutdatedLoaded), `src/ui/tools.rs` lines 99-113
- Impact: If tools list has 100 entries and outdated has 50, rendering still does 100 lookups on each frame. With multiple tabs rendering simultaneously, could cause frame drops.
- Fix approach: Pre-compute outdated status into tool model at load time, cache render state between frames, use interior mutability for lazy rendering.

**Unbounded Popup Search in Version Picker:**
- Issue: Version picker filtering (line 1469-1484 in app.rs) creates new SkimMatcherV2 and rescores entire version list on every keystroke without caching results.
- Files: `src/app.rs` lines 1469-1484, `src/app.rs` lines 770-773 (PopupSearchInput handler)
- Impact: With 50+ versions in picker, search lag becomes noticeable. No debounce on input.
- Fix approach: Memoize fuzzy results keyed by (query, version_list), only recompute on new query, add incremental filtering.

## Error Handling & Recovery

**Silent Error Drops in Async Operations:**
- Issue: Multiple `let _ = tx.send(...)` patterns (e.g., lines 257-258, 492-495, 566-573) silently discard send errors when action channel is closed. No logging or user feedback if async task fails to report results.
- Files: `src/app.rs` (50+ instances throughout), `src/event.rs` lines 20, 31, 43
- Impact: Operations may appear to hang if async task succeeds but channel is dead. User has no way to know why data isn't loading.
- Fix approach: Add error logging via `eprintln!` or tracing crate, create dedicated error channel, show error toasts to user, add timeout/retry for network operations.

**Weak Prune Output Parsing:**
- Issue: Prune dry-run parser (src/mise.rs lines 137-162) has three fallback patterns (split on '@', split on space, or assume whole line is tool) with no validation that parsed result makes sense.
- Files: `src/mise.rs` lines 137-162
- Impact: Malformed `mise prune` output could result in displaying garbage data to user, or worse, pruning unintended versions if format changes.
- Fix approach: Add unit tests for parser with known output samples, validate parsed results (non-empty tool and version), add version to prune command to ensure consistent format.

**Missing Bounds Checking in String Operations:**
- Issue: Tool name extraction in model.rs line 50 uses `rsplit('/').next().unwrap_or(&s.path)` which could panic if iterator is consumed unexpectedly (though safe here, pattern is fragile).
- Files: `src/model.rs` lines 50-52
- Impact: While currently safe due to `unwrap_or`, similar patterns throughout codebase could be vulnerability points.
- Fix approach: Use `.rsplit('/').next()` only in safe contexts, consider helper function `parse_source_name()`, add assertions in model construction.

**JSON Parsing Doesn't Handle Schema Drift:**
- Issue: Deserialization throughout (mise.rs lines 24-26, 31-33, etc.) fails completely if mise CLI output schema changes. No version checking or graceful degradation.
- Files: `src/mise.rs` lines 24-26, 31-33, 39-40, etc.
- Impact: If mise version changes output JSON structure, entire app breaks silently. User sees "Loading..." forever.
- Fix approach: Add schema version check to responses, implement compatibility layer for multiple versions, add detailed error messages showing actual output, consider using `serde` `#[serde(default)]` more aggressively.

## Fragile Areas

**Popup State Machine - Multiple Transitions:**
- Issue: Popup enum (lines 70-91 in app.rs) has manual state transitions throughout handle_action. Progress popup used as temporary state during async operations, transformed into ToolDetail or VersionPicker without strong guarantees.
- Files: `src/app.rs` lines 430-449 (ToolInfoLoaded state transition), `src/app.rs` lines 452-477 (VersionsLoaded), `src/ui/popup.rs` (rendering)
- Impact: Easy to show wrong popup state if async response arrives in wrong order or if transitions are missed. Confirm popup could incorrectly execute stale action if timing is off.
- Fix approach: Create explicit PopupState enum with clear transitions, add assertions for expected state changes, use TypeScript-style discriminated unions for state validation.

**Search Query Persistence Across Tab Switches:**
- Issue: `search_query` field (line 149 in app.rs) is global to App, not per-tab. Switching tabs while searching maintains same query across different data types with different search semantics.
- Files: `src/app.rs` lines 148-149, lines 1003-1036 (tab-specific rendering)
- Impact: User searches for "python" on Tools tab, switches to Registry tab, sees "python"-filtered Registry results. Confusing UX, state leak between tabs.
- Fix approach: Make search query per-tab, store in separate HashMap, clear when entering search mode, document intended search scope clearly.

**Doc Tab Scrolling Without Line Count Validation:**
- Issue: Doctor tab uses `doctor_scroll` (line 140) and `adjust_scroll` method (lines 1048-1055) but `filtered_doctor` is index list only. Line counting in scroll is done at render time from `info.lines().count()` (line 986).
- Files: `src/app.rs` lines 140, 1048-1055, 1016-1019, `src/ui/tools.rs` line 986
- Impact: Can't determine if scroll position is valid until render time. Could scroll past bounds after filtering.
- Fix approach: Cache line counts with data, validate scroll position when filter updates.

## Test Coverage Gaps

**No Unit Tests for Filter Logic:**
- Issue: Complex scoring/ranking in `update_filtered_*` methods untested. Fuzzy matcher behavior, score comparison, sorting order all live in logic without verification.
- Files: `src/app.rs` lines 1091-1300
- Impact: Regressions in search ranking go undetected, changes to sort behavior could silently break ordering, score tie-breaking behavior undefined.
- Fix approach: Extract filter logic to pure functions, add unit tests with known inputs/expected outputs, create test fixtures for various data sizes.

**No Integration Tests for Async Operations:**
- Issue: Async operations in app (install/uninstall/upgrade) never tested against mock mise. Action dispatch and side effects not validated.
- Files: `src/app.rs` lines 480-621
- Impact: Changes to action handling could break workflows silently, missing error cases in operation flows, no verification of action ordering.
- Fix approach: Create mock mise module for tests, write integration tests for install->refresh flow, test error recovery paths.

**Popup State Transitions Untested:**
- Issue: Complex popup state machine (VersionPicker -> Progress -> ToolDetail) with filters and selections not tested for correctness or edge cases.
- Files: `src/app.rs` lines 760-789 (PopupSearchInput/Backspace handlers), popup.rs rendering
- Impact: Edge cases like empty search results in version picker, rapid state transitions, or filter becoming inconsistent go undetected.
- Fix approach: Write tests for popup lifecycle, verify filter state after each transition, test edge cases (empty versions, special characters in search).

## Security Considerations

**Unsanitized mise CLI Command Output:**
- Issue: Tool names, versions, and paths captured directly from `mise` CLI output without sanitization. Displayed in TUI which uses these for building new commands.
- Files: `src/mise.rs` (all fetch functions), `src/model.rs` (data parsing)
- Impact: If user installs tool with special characters in name/version, could cause undefined behavior or injection if name is used in subsequent commands (e.g., `mise install {tool}@{version}`).
- Fix approach: Validate/sanitize tool names and versions against regex, escape properly when building commands, add integration test with special character names.

**No Input Validation for Confirm Dialog:**
- Issue: Confirm actions (lines 793-930 in app.rs) dispatch directly from extracted data without re-validation. Path for `TrustConfig` action not checked before passing to `mise trust`.
- Files: `src/app.rs` lines 882-896 (TrustConfig confirm handler)
- Impact: Theoretically could lead to trusting unexpected paths if state gets corrupted or race condition occurs.
- Fix approach: Re-validate path/tool name at confirm time, add assertions for expected state, use newtype wrappers for trusted paths.

**Channel Endpoints Exposed Without Auth:**
- Issue: Action channel is cloned and passed to async spawns without access control. In theory, multiple uncoordinated sources could spam commands.
- Files: `src/app.rs` lines 184, 253-309, throughout handle_action
- Impact: Low risk in single-user TUI context, but pattern could be dangerous in multi-component architecture.
- Fix approach: Consider action enum variants, rate limiting, or audit logging of commands (if architecture ever changes).

## Dependency Risks

**Fuzzy-Matcher 0.3 - No Recent Updates:**
- Issue: fuzzy-matcher crate pinned to 0.3 without patch version flexibility.
- Files: `Cargo.toml` line 21
- Impact: Could miss security patches or bug fixes if newer patch released, unclear if active maintenance continues.
- Fix approach: Update to latest patch version, enable caret version (`^0.3`) to get patches automatically, check GitHub for maintainer activity.

**Tokio Full Features - Overkill:**
- Issue: `tokio = { version = "1", features = ["full"] }` pulls in all tokio runtime features even though only async runtime + mpsc needed.
- Files: `Cargo.toml` line 19
- Impact: Larger binary, unnecessary dependencies, harder to audit what's actually used.
- Fix approach: Specify only needed features: `["rt", "sync", "time", "macros"]`, verify app still runs.

**No Lock File Committed:**
- Issue: No `Cargo.lock` visible in repo (if building library) or if committed, dependency versions not frozen.
- Files: (Repository level)
- Impact: Builds could be non-reproducible, transitive dependency updates could silently change behavior.
- Fix approach: Commit `Cargo.lock` if not already present, document dependency update policy.

## Missing Features / Anti-patterns

**No Offline Mode or Cache:**
- Issue: Every refresh fetches all data from `mise` CLI, which must be available and responsive. No caching or stale-while-revalidate.
- Files: `src/app.rs` lines 253-309
- Impact: If mise CLI hangs or network is slow, app is unresponsive. Previous data is lost on filter/refresh cycles.
- Fix approach: Add optional disk cache layer, implement last-known-good data fallback, add operation timeout + cached data indicator.

**No Keybinding Customization:**
- Issue: All keybindings hard-coded in `main.rs` remap functions with no config file support.
- Files: `src/main.rs` lines 64-119
- Impact: User cannot remap keys to personal preference, conflicts with other tools impossible to resolve.
- Fix approach: Load keybindings from config file (~/.misetui.toml), implement binding validation, provide sensible defaults.

**No Progress Indication for Long Operations:**
- Issue: Install/upgrade operations show generic "Installing..." message with spinner but no progress percentage or ETA.
- Files: `src/ui/popup.rs` lines 145+ (Progress rendering)
- Impact: User doesn't know how long operation will take, unclear if it's stalled.
- Fix approach: Parse mise stderr for progress info, show percentage complete, add timeout warning if taking too long.

**Limited Error Detail to User:**
- Issue: Operation errors (lines 950-952 in app.rs) shown as "Error: {msg}" toast with no way to see full details or retry.
- Files: `src/app.rs` lines 950-952 (OperationFailed handler)
- Impact: User can't debug issues, retrying requires full refresh.
- Fix approach: Store error details, add expandable error popup with full message, add retry button for transient failures.

---

*Concerns audit: 2026-02-23*
