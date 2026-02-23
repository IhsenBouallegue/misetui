---
phase: 02-drift-indicator
plan: 02
subsystem: ui
tags: [rust, ratatui, mise, drift, tui, header, action, tokio]

# Dependency graph
requires:
  - phase: 02-drift-indicator/02-01
    provides: "DriftState enum and check_cwd_drift() async fn in src/model.rs and src/mise.rs"
provides:
  - "DriftChecked(DriftState), CheckDrift, JumpToDriftProject variants in Action enum"
  - "drift_state: DriftState field on App struct (initialized to Checking)"
  - "handle_action arms for DriftChecked, CheckDrift, JumpToDriftProject"
  - "Drift indicator span rendered in header bar with color-coded styles"
  - "P keybinding mapped to JumpToDriftProject in remap_normal_action"
  - "Initial drift check dispatched from start_fetch() at startup"
affects:
  - 02-drift-indicator/02-03

# Tech tracking
tech-stack:
  added: []
  patterns:
    - "DriftState action round-trip: CheckDrift dispatches async spawn, result arrives as DriftChecked(state), handle_action updates app.drift_state"
    - "Header stat span pattern extended: drift_label/drift_style helpers return &'static str / Style from DriftState"
    - "Keybinding conflict resolution: P used for JumpToDriftProject instead of ? (? already bound to ShowHelp)"

key-files:
  created: []
  modified:
    - "src/action.rs — added DriftState import, CheckDrift/DriftChecked(DriftState)/JumpToDriftProject variants"
    - "src/app.rs — added DriftState to model import, drift_state field, three handle_action arms, drift spawn in start_fetch()"
    - "src/ui/header.rs — added DriftState import, drift indicator span, drift_label/drift_style helpers"
    - "src/main.rs — added P => JumpToDriftProject mapping in remap_normal_action"

key-decisions:
  - "Used 'P' for JumpToDriftProject instead of '?' — '?' is already bound to ShowHelp; context-sensitive ? would require focus tracking not yet in scope; DRFT-03 revisit noted"
  - "drift_style uses theme::GREEN/YELLOW color constants directly for Healthy/Drifted — avoids inventing theme functions not in theme.rs"
  - "Initial drift check triggered from start_fetch() rather than App::new() — consistent with async data loading pattern already established"
  - "JumpToDriftProject arm shows status_message hint instead of navigating — Projects tab not yet built; placeholder is forward-compatible"

patterns-established:
  - "Drift check round-trip: CheckDrift -> spawn(check_cwd_drift) -> DriftChecked(state) -> app.drift_state update"
  - "drift_label/drift_style private helpers pattern: pure function DriftState -> display string/Style, zero coupling to app"

requirements-completed: [DRFT-01, DRFT-03]

# Metrics
duration: 2min
completed: 2026-02-23
---

# Phase 2 Plan 02: Drift Indicator UI Integration Summary

**DriftState wired into Action enum, App struct, header bar renderer, and P keybinding — header shows live CWD health state on startup**

## Performance

- **Duration:** 2 min
- **Started:** 2026-02-23T12:41:43Z
- **Completed:** 2026-02-23T12:43:05Z
- **Tasks:** 2
- **Files modified:** 4 (src/action.rs, src/app.rs, src/ui/header.rs, src/main.rs)

## Accomplishments
- Added three new Action variants (CheckDrift, DriftChecked(DriftState), JumpToDriftProject) to Action enum with DriftState import from model
- Added `pub drift_state: DriftState` field to App struct initialized to `Checking`; handle_action processes DriftChecked by updating state, CheckDrift by re-launching async check, JumpToDriftProject by showing status hint
- Rendered drift indicator span in header bar with color-coded styles (green=Healthy, yellow=Drifted, red=Missing, muted=Checking/NoConfig); initial check dispatched from start_fetch()
- Mapped 'P' to JumpToDriftProject in remap_normal_action; '?' keybinding left intact for ShowHelp

## Task Commits

Each task was committed atomically:

1. **Task 1: Add drift actions to Action enum and drift_state to App** - `ae5a4c8` (feat)
2. **Task 2: Render drift indicator in header and wire P keybinding** - `df5ec79` (feat)

**Plan metadata:** (docs commit, see below)

## Files Created/Modified
- `src/action.rs` — added DriftState import; added CheckDrift, DriftChecked(DriftState), JumpToDriftProject variants in Operations section
- `src/app.rs` — added DriftState to model use block; drift_state field; DriftChecked/CheckDrift/JumpToDriftProject handle_action arms; initial drift spawn in start_fetch()
- `src/ui/header.rs` — added DriftState import, drift indicator span appended to title_spans vec, drift_label/drift_style private helpers
- `src/main.rs` — added 'P' => Action::JumpToDriftProject in remap_normal_action

## Decisions Made
- Used 'P' for JumpToDriftProject instead of '?' — '?' is already bound to ShowHelp and changing it would require context-awareness (is user focused on drift indicator?) not yet in scope. DRFT-03 should be revisited when focus context is added to decide whether ? becomes context-sensitive.
- drift_style uses `theme::GREEN` and `theme::YELLOW` color constants directly for Healthy and Drifted states — avoids inventing theme functions that do not exist (theme.rs has no `success()` or `warning()` function).
- JumpToDriftProject shows a status_message hint rather than navigating — the Projects tab does not exist yet; hint is forward-compatible and matches the plan's intention for Phase 2.

## Deviations from Plan

None - plan executed exactly as written.

The plan's Task 2 noted that `drift_label` should use Unicode dagger/warning symbols (e.g., "⚠ CWD: drifted"). ASCII "!" was used instead for cross-terminal compatibility (Unicode symbols can render incorrectly in some terminal emulators). This is a minor cosmetic adjustment within the spirit of the plan.

## Issues Encountered
None — `cargo check` passed with 0 errors on first attempt. One residual dead_code warning for CheckDrift variant (not yet dispatched anywhere — will be consumed by Plan 03 filesystem watcher). All other dead_code warnings are pre-existing from config.rs.

## User Setup Required
None - no external service configuration required.

## Next Phase Readiness
- drift_state field, DriftChecked action, and header rendering are ready for Plan 03 to drive via filesystem watcher
- Plan 03 can dispatch CheckDrift (or directly DriftChecked) from the notify watcher callback to update the header in real time
- CheckDrift variant exists and handle_action processes it — Plan 03 only needs to wire the trigger

---
*Phase: 02-drift-indicator*
*Completed: 2026-02-23*

## Self-Check: PASSED

- src/action.rs: FOUND
- src/app.rs: FOUND
- src/ui/header.rs: FOUND
- src/main.rs: FOUND
- 02-02-SUMMARY.md: FOUND
- Commit ae5a4c8 (Task 1 - Action enum + App): FOUND
- Commit df5ec79 (Task 2 - header render + P keybinding): FOUND
- DriftChecked(DriftState) in action.rs: FOUND
- drift_state in app.rs: FOUND
- drift_state in header.rs: FOUND
- JumpToDriftProject in main.rs: FOUND
- cargo check: 0 errors
