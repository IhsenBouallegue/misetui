---
phase: 02-drift-indicator
plan: 01
subsystem: data-model
tags: [rust, ratatui, mise, drift, notify, inotify, async, tokio]

# Dependency graph
requires: []
provides:
  - "DriftState enum (Checking, Healthy, Drifted, Missing, NoConfig) in src/model.rs"
  - "check_cwd_drift() async function in src/mise.rs"
  - "notify v6 crate in Cargo.toml for filesystem watching"
affects:
  - 02-drift-indicator/02-02
  - 02-drift-indicator/02-03

# Tech tracking
tech-stack:
  added:
    - "notify = \"6\" (v6.1.1) — filesystem watcher, uses inotify on Linux"
  patterns:
    - "DriftState enum as primary CWD health signal — exit code + keyword tiebreaker pattern"
    - "check_cwd_drift() shells out to `mise status`, maps exit code to typed enum variant"

key-files:
  created: []
  modified:
    - "src/model.rs — added DriftState enum with 5 variants"
    - "src/mise.rs — added check_cwd_drift() pub async fn and DriftState import"
    - "Cargo.toml — added notify = \"6\" dependency"
    - "Cargo.lock — locked notify v6.1.1 and transitive deps"

key-decisions:
  - "Used notify v6 (not v8) as specified in plan — resolves without feature flags on Linux (inotify automatic)"
  - "check_cwd_drift() trusts mise status exit code as primary signal; uses stdout/stderr keyword matching only as tiebreaker for Missing vs Drifted"
  - "No Deserialize derive on DriftState — it is not deserialized from JSON, only produced by check_cwd_drift()"
  - "macos_fsevent feature omitted on Linux (unnecessary; inotify is the automatic backend)"

patterns-established:
  - "DriftState variants: Checking (in-flight), Healthy (ok), Drifted (mismatch), Missing (not installed), NoConfig (no config applies)"
  - "Async health-check pattern: run CLI command, map exit code + output to typed enum, return Result<State, String>"

requirements-completed: [DRFT-01, DRFT-02]

# Metrics
duration: 1min
completed: 2026-02-23
---

# Phase 2 Plan 01: Drift Indicator Data Foundation Summary

**DriftState enum and check_cwd_drift() async function backed by `mise status`, plus notify v6 crate for filesystem watching in subsequent plans**

## Performance

- **Duration:** 1 min
- **Started:** 2026-02-23T12:38:16Z
- **Completed:** 2026-02-23T12:39:25Z
- **Tasks:** 2
- **Files modified:** 4 (src/model.rs, src/mise.rs, Cargo.toml, Cargo.lock)

## Accomplishments
- Added `DriftState` enum to `src/model.rs` with 5 variants covering all CWD health outcomes (Checking, Healthy, Drifted, Missing, NoConfig), no JSON deserialization needed
- Added `pub async fn check_cwd_drift() -> Result<DriftState, String>` to `src/mise.rs` that shells out to `mise status`, uses exit code as primary signal, keyword tiebreaker for Missing vs Drifted
- Added `notify = "6"` to Cargo.toml (resolved to v6.1.1 using inotify backend on Linux); provides the filesystem-watching foundation for Plan 03

## Task Commits

Each task was committed atomically:

1. **Task 1: Add DriftState enum to src/model.rs** - `c96c842` (feat)
2. **Task 2: Add check_cwd_drift() to mise.rs and notify to Cargo.toml** - `0a398b5` (feat)

**Plan metadata:** (docs commit, see below)

## Files Created/Modified
- `src/model.rs` — appended DriftState enum (5 variants, derives Debug/Clone/Copy/PartialEq/Eq)
- `src/mise.rs` — added DriftState to use block, appended check_cwd_drift() at end of file
- `Cargo.toml` — added `notify = "6"` under [dependencies]
- `Cargo.lock` — locked notify v6.1.1 and 24 transitive packages (crossbeam, inotify, mio, walkdir, etc.)

## Decisions Made
- Used notify v6 as plan specified (not v8 which was available); v6 resolves without the `macos_fsevent` feature on Linux since inotify is the automatic backend
- check_cwd_drift() trusts `mise status` exit code as the primary signal for health state — keyword scanning in stdout/stderr is only used to distinguish Missing from Drifted in the non-zero exit case
- DriftState derives `Copy` in addition to Clone for ergonomic use when passed across task boundaries

## Deviations from Plan

None - plan executed exactly as written.

The only minor adaptation was omitting the `macos_fsevent` feature from the notify dependency (plan noted it as "recommended backend for macOS") since the build environment is Linux; this matches the plan's own parenthetical that "on Linux notify uses inotify automatically."

## Issues Encountered
None — `cargo check` passed with 0 errors on first attempt. Two dead_code warnings appeared (DriftState and check_cwd_drift not yet consumed by app code); these are expected and will resolve in Plans 02/03.

## User Setup Required
None - no external service configuration required.

## Next Phase Readiness
- DriftState type and check_cwd_drift() are ready for Plans 02 and 03 to consume in parallel
- Plan 02 can add drift_state field to App struct and wire the periodic refresh
- Plan 03 can use the notify crate to watch .mise.toml for filesystem changes

---
*Phase: 02-drift-indicator*
*Completed: 2026-02-23*

## Self-Check: PASSED

- src/model.rs: FOUND
- src/mise.rs: FOUND
- Cargo.toml: FOUND
- 02-01-SUMMARY.md: FOUND
- Commit c96c842 (Task 1 - DriftState enum): FOUND
- Commit 0a398b5 (Task 2 - check_cwd_drift + notify): FOUND
- pub enum DriftState in model.rs: FOUND (all 5 variants including NoConfig)
- pub async fn check_cwd_drift() in mise.rs: FOUND
- notify in Cargo.toml: FOUND
- cargo check: 0 errors, 2 expected dead_code warnings
