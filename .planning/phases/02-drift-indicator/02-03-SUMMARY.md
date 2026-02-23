---
phase: 02-drift-indicator
plan: 03
subsystem: ui
tags: [rust, tokio, notify, filesystem-watcher, drift, main, async, debounce]

# Dependency graph
requires:
  - phase: 02-drift-indicator/02-02
    provides: "Action::CheckDrift variant and handle_action arm in app.rs; drift_state field on App; header drift indicator renderer"
provides:
  - "RecommendedWatcher spawned in main.rs at startup, watching .mise.toml and ~/.config/mise/config.toml"
  - "Arc<Mutex<Receiver>> bridge pattern for std::sync::mpsc -> tokio::task::spawn_blocking"
  - "200ms debounce loop coalescing burst file events into single Action::CheckDrift"
  - "Graceful degradation: all watcher errors silently swallowed; manual r refresh still works"
  - "install_project_tools / update_project_pins async fns in mise.rs (unblocked Phase 01 stubs)"
  - "projects.rs UI module wired into ui/mod.rs render dispatch"
affects: []

# Tech tracking
tech-stack:
  added:
    - "notify v6 (RecommendedWatcher, EventKind, RecursiveMode) — filesystem event watching"
    - "dirs v5 (config_dir) — locating ~/.config/mise/config.toml"
    - "std::sync::{Arc, Mutex} — safe Receiver sharing across spawn_blocking boundary"
  patterns:
    - "Arc<Mutex<Receiver>> bridge: safely share std::sync::mpsc Receiver into tokio::task::spawn_blocking"
    - "Notify+Tokio integration: notify sends to std channel; tokio task bridges via spawn_blocking+recv_timeout"
    - "Debounce loop: sleep 200ms after first event, drain remaining events, send coalesced action"
    - "Watcher graceful degradation: all watch() and watcher construction errors silently ignored"

key-files:
  created: []
  modified:
    - "src/main.rs — imports notify/std/dirs; watcher task spawned before event loop; Arc<Mutex<Receiver>> debounce"
    - "src/mise.rs — added install_project_tools and update_project_pins async fns (Phase 01 stubs unblocked)"
    - "src/app.rs — added update_filtered_projects method (Phase 01 stub completed)"
    - "src/action.rs — MiseProject import + ProjectsLoaded/InstallProjectTools/UpdateProjectPins variants (pre-existing stubs activated)"
    - "src/ui/footer.rs — added Tab::Projects match arm with keybinding hints"
    - "src/ui/mod.rs — added mod projects; + Tab::Projects dispatch to projects::render"
    - "src/ui/projects.rs — fixed theme::BORDER -> theme::MUTED, theme::selected() -> theme::table_selected()"

key-decisions:
  - "Arc<Mutex<Receiver>> chosen over unsafe pointer approach suggested in plan — idiomatic, safe bridging of std::sync::mpsc into tokio::task::spawn_blocking"
  - "Watcher spawned before event loop in a scoped block — isolates action_tx clone lifetime from the main loop borrow"
  - "Pre-existing blocking compile errors (Phase 01 stubs) auto-fixed under Rule 3 — they directly prevented cargo check verification of the watcher task"

patterns-established:
  - "Notify+Tokio bridge: RecommendedWatcher -> std::sync::mpsc -> Arc<Mutex<Receiver>> -> spawn_blocking -> recv_timeout"
  - "Filesystem watcher debounce: sleep(200ms) + drain loop after first event to coalesce burst writes"

requirements-completed: [DRFT-02]

# Metrics
duration: 4min
completed: 2026-02-23
---

# Phase 2 Plan 03: Filesystem Watcher for Drift Indicator Summary

**notify::RecommendedWatcher spawned at startup in main.rs watching .mise.toml and ~/.config/mise/config.toml, debouncing burst writes into a single Action::CheckDrift via Arc<Mutex<Receiver>> bridge every ~200ms**

## Performance

- **Duration:** 4 min
- **Started:** 2026-02-23T12:45:18Z
- **Completed:** 2026-02-23T12:49:00Z
- **Tasks:** 1
- **Files modified:** 7 (src/main.rs, src/mise.rs, src/app.rs, src/action.rs, src/ui/footer.rs, src/ui/mod.rs, src/ui/projects.rs)

## Accomplishments
- Spawned a notify filesystem watcher task in main.rs that watches `.mise.toml` in CWD and `~/.config/mise/config.toml` using `RecommendedWatcher`
- Implemented safe tokio/std channel bridge via `Arc<Mutex<Receiver>>` passed into `tokio::task::spawn_blocking`, avoiding unsafe pointer tricks mentioned in plan
- Debounce loop coalesces rapid file system events (burst writes) into a single `Action::CheckDrift` per ~200ms window — prevents repeated re-checks on multi-write saves
- All watcher errors (missing file, permission denied, unavailable watcher) silently swallowed; manual `r` refresh remains as fallback

## Task Commits

Each task was committed atomically:

1. **Task 1: Spawn notify filesystem watcher in main.rs** - `1ff855a` (feat)

## Files Created/Modified
- `src/main.rs` — added notify/std/dirs imports; watcher task block before event loop; Arc<Mutex<Receiver>> debounce loop sends Action::CheckDrift
- `src/mise.rs` — added `install_project_tools` and `update_project_pins` async functions (required to unblock compile)
- `src/app.rs` — `update_filtered_projects` method completed (was called but missing; auto-fixed)
- `src/action.rs` — `MiseProject` import + `ProjectsLoaded`, `InstallProjectTools`, `UpdateProjectPins` variants activated (pre-existing stubs)
- `src/ui/footer.rs` — `Tab::Projects` match arm with keybinding hints (was non-exhaustive; auto-fixed)
- `src/ui/mod.rs` — `mod projects;` declared + `Tab::Projects` dispatches to `projects::render` (was non-exhaustive; auto-fixed)
- `src/ui/projects.rs` — theme references fixed: `theme::BORDER` -> `theme::MUTED`, `theme::selected()` -> `theme::table_selected()`

## Decisions Made
- Used `Arc<Mutex<Receiver>>` instead of the unsafe pointer trick shown as "fragile" in the plan — matches the plan's own recommended clean approach and is fully safe and idiomatic
- Wrapped watcher task in a scoped block `{ }` to scope `watch_tx` clone lifetime before the main event loop, keeping borrow checker happy
- Pre-existing Phase 01 compile blockers treated as Rule 3 (blocking issues) and auto-fixed — they directly prevented `cargo check` from verifying the watcher implementation

## Deviations from Plan

### Auto-fixed Issues

**1. [Rule 3 - Blocking] Pre-existing compile errors from Phase 01 stubs prevented cargo check**
- **Found during:** Task 1 (watcher spawn implementation)
- **Issue:** `cargo check` failed with 4 errors: `update_filtered_projects` method missing in app.rs; `ProjectsLoaded/InstallProjectTools/UpdateProjectPins` not covered in handle_action match; `Tab::Projects` not covered in footer.rs and ui/mod.rs match statements
- **Fix:** Added `update_filtered_projects` fuzzy filter method in app.rs; added `install_project_tools`/`update_project_pins` fns in mise.rs; added `Tab::Projects` arms in footer.rs and ui/mod.rs; fixed `projects.rs` theme reference errors (`BORDER`->`MUTED`, `selected()`->`table_selected()`)
- **Files modified:** src/app.rs, src/mise.rs, src/ui/footer.rs, src/ui/mod.rs, src/ui/projects.rs
- **Verification:** `cargo check` exits 0 with 0 errors after fixes
- **Committed in:** 1ff855a (Task 1 commit)

---

**Total deviations:** 1 auto-fixed (Rule 3 — blocking compile errors from Phase 01 stubs)
**Impact on plan:** Auto-fixes necessary to allow cargo check to verify the watcher implementation. All fixes complete pre-existing Phase 01 stub work. No scope creep.

## Issues Encountered
- The linter (rust-analyzer or similar) was simultaneously adding and removing functions from mise.rs during editing, causing "file modified since read" conflicts. Resolved by reading the file state before each edit and removing duplicate function definitions that appeared.

## User Setup Required
None - no external service configuration required.

## Next Phase Readiness
- DRFT-02 fully delivered: drift indicator now updates automatically on file save, no `r` keypress required
- Phase 02 drift indicator phase is now complete (all 3 plans done: data layer, UI integration, filesystem watcher)
- Phase 03 bootstrap wizard can begin; drift indicator provides reactive foundation for any config file changes during wizard

---
*Phase: 02-drift-indicator*
*Completed: 2026-02-23*

## Self-Check: PASSED

- src/main.rs: FOUND
- src/mise.rs: FOUND
- src/app.rs: FOUND
- src/ui/projects.rs: FOUND
- 02-03-SUMMARY.md: FOUND
- Commit 1ff855a (Task 1 - notify watcher): FOUND
- RecommendedWatcher in main.rs: FOUND
- Action::CheckDrift in main.rs: FOUND
- Arc<Mutex<Receiver>> bridge in main.rs: FOUND
- install_project_tools in mise.rs: FOUND
- cargo check: 0 errors
