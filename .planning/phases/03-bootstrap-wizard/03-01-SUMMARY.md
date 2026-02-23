---
phase: 03-bootstrap-wizard
plan: 01
subsystem: ui
tags: [wizard, model, mise, detection, migration, toml]

# Dependency graph
requires:
  - phase: 02-drift-indicator
    provides: DriftState, MiseProject, scan_projects — established the mise.rs async patterns this plan follows
provides:
  - WizardState struct (multi-step wizard state: target_dir, step, tools, selected, preview_content, write_agent_files, preview_scroll)
  - WizardStep enum (Detecting/Review/Preview/Writing)
  - DetectedTool struct (name/version/source/enabled/installed)
  - detect_project_tools(dir: &str) -> Vec<DetectedTool> (async, cross-references mise ls -J)
  - migrate_legacy_pins(dir: &str) -> Vec<DetectedTool> (sync, parses .tool-versions)
affects: [03-bootstrap-wizard-plan02, 03-bootstrap-wizard-plan03]

# Tech tracking
tech-stack:
  added: []
  patterns:
    - "Filesystem indicator → tool detection: package.json→node, Cargo.toml→rust, pyproject.toml/requirements.txt→python, go.mod→go, Gemfile→ruby, composer.json→php"
    - "Priority hierarchy: .tool-versions (lowest) < filesystem indicators < explicit legacy pin files"
    - "async detect fn + sync migrate fn: async function calls sync internally after filesystem I/O"
    - "Cross-reference pattern: run_mise(&[\"ls\", \"-J\"]) to populate installed bool on detected tools"

key-files:
  created: []
  modified:
    - src/model.rs
    - src/mise.rs

key-decisions:
  - "DetectedTool::installed populated by cross-referencing mise ls -J output (not a separate mise call per tool)"
  - "Priority order: .tool-versions lowest, then filesystem indicators, then standalone legacy pins (.nvmrc/.python-version/.ruby-version)"
  - "migrate_legacy_pins() is synchronous (pure fs I/O); detect_project_tools() is async (calls mise ls -J)"
  - "Version fallbacks: node=lts, python/go/ruby/php=latest, rust=stable when no explicit pin found"

patterns-established:
  - "Wizard foundation pattern: pure data types in model.rs, async detection logic in mise.rs — Plan 02 imports WizardState and calls detect_project_tools"
  - "Tool toggle pattern: DetectedTool.enabled=true by default, toggled in Review step (Plan 02)"

requirements-completed: [BOOT-01, BOOT-02, BOOT-03]

# Metrics
duration: 1min
completed: 2026-02-23
---

# Phase 03 Plan 01: Bootstrap Wizard Data Model and Detection Logic Summary

**WizardState/WizardStep/DetectedTool model types and async detect_project_tools() + sync migrate_legacy_pins() functions for Bootstrap Wizard foundation**

## Performance

- **Duration:** 1 min
- **Started:** 2026-02-23T15:10:19Z
- **Completed:** 2026-02-23T15:11:44Z
- **Tasks:** 2
- **Files modified:** 2

## Accomplishments
- Added DetectedTool struct with name/version/source/enabled/installed fields to model.rs
- Added WizardState multi-step state struct and WizardStep enum to model.rs
- Implemented async detect_project_tools() scanning 7 filesystem indicators with mise ls -J cross-reference for installed status
- Implemented sync migrate_legacy_pins() parsing .tool-versions format for legacy tool version migration

## Task Commits

Each task was committed atomically:

1. **Task 1: Add WizardState model types to model.rs** - `fca8994` (feat)
2. **Task 2: Add detect_project_tools() and migrate_legacy_pins() to mise.rs** - `a98c641` (feat)

**Plan metadata:** (docs commit follows)

## Files Created/Modified
- `src/model.rs` - Added DetectedTool, WizardState, WizardStep types
- `src/mise.rs` - Added detect_project_tools() async fn and migrate_legacy_pins() sync fn; added DetectedTool to model import

## Decisions Made
- DetectedTool::installed is populated by cross-referencing the existing `mise ls -J` output rather than spawning a separate subprocess per tool, maintaining consistency with existing fetch_tools() pattern
- migrate_legacy_pins() stays synchronous since it is pure filesystem I/O called inside the async detect_project_tools()
- Version fuzzy matching for "latest"/"stable"/"lts" special values treated as always-installed in the cross-reference step

## Deviations from Plan

None - plan executed exactly as written.

## Issues Encountered
None - cargo check passes with zero errors; only expected dead_code warnings for new functions not yet wired into app.rs (done by Plan 02).

## User Setup Required
None - no external service configuration required.

## Next Phase Readiness
- Plan 02 can now import WizardState/DetectedTool from model.rs and call detect_project_tools()/migrate_legacy_pins() from mise.rs
- All BOOT-01/02/03 foundation types present and compiling
- No blockers

---
*Phase: 03-bootstrap-wizard*
*Completed: 2026-02-23*
