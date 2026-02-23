---
phase: 01-projects-tab
plan: 05
subsystem: ui
tags: [ratatui, popup, config, toml, serde]

# Dependency graph
requires:
  - phase: 01-projects-tab
    provides: Projects tab renderer and app state from plans 01-03

provides:
  - In-app scan config editing popup (Popup::ScanConfig variant)
  - MisetuiConfig::save() writing TOML to ~/.config/misetui/config.toml
  - OpenScanConfig and SaveScanConfig actions
  - remap_scan_config_action() for popup-mode key routing
  - render_scan_config() popup renderer with dir list, depth control, add mode

affects:
  - phase-03 (bootstrap wizard may interact with config saving pattern)

# Tech tracking
tech-stack:
  added: []
  patterns:
    - ScanConfig popup intercept block before main handle_action match
    - Popup-mode key remapping via remap_scan_config_action + is_scan_config_active guard
    - Config save uses Serialize + toml::to_string_pretty + std::fs::write

key-files:
  created: []
  modified:
    - src/action.rs
    - src/app.rs
    - src/config.rs
    - src/main.rs
    - src/ui/popup.rs

key-decisions:
  - "ScanConfig popup intercept block placed before main match in handle_action — consistent with VersionPicker intercept pattern via move_selection"
  - "remap_scan_config_action routes 'd'->UninstallTool, 'a'->InstallTool, 'q'->CancelPopup; other chars pass through for typing in add mode"
  - "render_scan_config added to popup.rs alongside Task 1 commit to fix non-exhaustive match compile error (Rule 3 auto-fix)"

patterns-established:
  - "Popup intercept pattern: add if-let ScanConfig block before main match for popup-specific key handling"
  - "Config persistence: MisetuiConfig::save() using toml::to_string_pretty, creates parent dirs if missing"

requirements-completed: [PROJ-08]

# Metrics
duration: 3min
completed: 2026-02-23
---

# Phase 01 Plan 05: Scan Config Popup Summary

**In-app scan directory and max_depth editor popup triggered by 'c' on Projects tab, saving to ~/.config/misetui/config.toml with immediate rescan**

## Performance

- **Duration:** 3 min
- **Started:** 2026-02-23T00:06:46Z
- **Completed:** 2026-02-23T00:09:34Z
- **Tasks:** 2
- **Files modified:** 5

## Accomplishments
- Added `Popup::ScanConfig` variant with full editing state (dirs list, selected index, adding mode, new_dir buffer, max_depth)
- Implemented `MisetuiConfig::save()` with TOML serialization and directory creation
- Added full event routing: `is_scan_config_active` guard, `remap_scan_config_action`, ScanConfig intercept block in `handle_action`
- Rendered interactive popup: max_depth row with -/+ controls, highlighted dir list, add-mode text input, hint bar

## Task Commits

Each task was committed atomically:

1. **Task 1: Add ScanConfig popup state, actions, config save, and event routing** - `0da8e88` (feat)
2. **Task 2: Render scan config popup and update help text** - `f58363d` (feat)

**Plan metadata:** (committed with state updates)

## Files Created/Modified
- `src/action.rs` - Added OpenScanConfig and SaveScanConfig action variants
- `src/app.rs` - Added Popup::ScanConfig variant, ScanConfig intercept block, OpenScanConfig/SaveScanConfig handlers
- `src/config.rs` - Added Serialize derive to MisetuiConfig, added pub fn save()
- `src/main.rs` - Added 'c' binding, is_scan_config_active(), remap_scan_config_action(), wired into event loop
- `src/ui/popup.rs` - Added ScanConfig render arm, render_scan_config() function, 'c' line in help

## Decisions Made
- ScanConfig intercept block placed at top of `handle_action` before main match — same pattern used by VersionPicker navigation in `move_selection`
- `render_scan_config` was implemented in the same session as Task 1 state changes to fix non-exhaustive match compile errors; committed as Task 2

## Deviations from Plan

### Auto-fixed Issues

**1. [Rule 3 - Blocking] Implemented popup renderer alongside Task 1 to fix compile errors**
- **Found during:** Task 1 build verification
- **Issue:** Adding `Popup::ScanConfig` variant caused non-exhaustive match errors in both `src/app.rs` (Confirm handler) and `src/ui/popup.rs` (render match) — build failed
- **Fix:** Added `Popup::ScanConfig { .. } => {}` arm to Confirm handler; implemented full `render_scan_config()` in popup.rs (Task 2 content) so build succeeds after Task 1 changes
- **Files modified:** src/app.rs, src/ui/popup.rs
- **Verification:** `cargo build` exits 0 after all changes
- **Committed in:** f58363d (Task 2 commit)

---

**Total deviations:** 1 auto-fixed (1 blocking compile issue)
**Impact on plan:** The Task 2 implementation was needed to unblock Task 1 compilation — tasks were effectively implemented together and committed separately. No scope creep.

## Issues Encountered
None beyond the expected non-exhaustive match compile errors resolved by completing both tasks together.

## User Setup Required
None - no external service configuration required. Config file is created automatically on first save.

## Next Phase Readiness
- Projects tab feature set complete (scan, drift, config editing)
- Phase 01 all 5 plans done
- Phase 03 (Bootstrap Wizard) can build on MisetuiConfig::save() pattern

## Self-Check: PASSED
- src/action.rs: FOUND OpenScanConfig, SaveScanConfig
- src/app.rs: FOUND Popup::ScanConfig variant
- src/config.rs: FOUND pub fn save
- src/main.rs: FOUND is_scan_config_active, remap_scan_config_action, 'c' binding
- src/ui/popup.rs: FOUND render_scan_config
- Commit 0da8e88: FOUND
- Commit f58363d: FOUND

---
*Phase: 01-projects-tab*
*Completed: 2026-02-23*
