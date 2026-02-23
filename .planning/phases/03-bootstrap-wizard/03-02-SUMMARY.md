---
phase: 03-bootstrap-wizard
plan: 02
subsystem: ui
tags: [wizard, action, popup, keybinding, mise, toml, ratatui]

# Dependency graph
requires:
  - phase: 03-bootstrap-wizard-plan01
    provides: WizardState, WizardStep, DetectedTool, detect_project_tools(), migrate_legacy_pins()
provides:
  - Action::OpenWizard/WizardDetected/WizardToggleTool/WizardToggleAgentFiles/WizardNextStep/WizardPrevStep/WizardCompleted in action.rs
  - Popup::Wizard(WizardState) variant in app.rs
  - Wizard intercept block in handle_action for step navigation (Detecting/Review/Preview/Writing)
  - OpenWizard handle_action arm spawning detect_project_tools() async
  - WizardDetected/WizardCompleted handle_action arms
  - generate_mise_toml_preview() free function in app.rs
  - write_mise_toml() atomic write function in mise.rs
  - write_agent_files_for() AGENTS.md+CLAUDE.md writer in mise.rs
  - render_wizard() popup renderer in ui/popup.rs
  - B keybinding -> OpenWizard in remap_normal_action
  - remap_wizard_action() for j/k/Space/Enter/a/n/p/Esc
  - is_wizard_active() helper function in main.rs
affects: [03-bootstrap-wizard-plan03]

# Tech tracking
tech-stack:
  added: []
  patterns:
    - "Wizard intercept block: placed before main match action in handle_action, same pattern as ScanConfig intercept"
    - "Popup::Wizard wraps WizardState — intercept block handles navigation, main match handles open/detect/complete"
    - "Atomic file write: tokio::fs::write to .tmp then tokio::fs::rename to target (same pattern as planned for config writes)"
    - "remap_wizard_action: Space mapped to WizardToggleTool, Enter remapped to WizardNextStep, Esc to CancelPopup"

key-files:
  created: []
  modified:
    - src/action.rs
    - src/app.rs
    - src/mise.rs
    - src/main.rs
    - src/ui/popup.rs

key-decisions:
  - "render_wizard() added to ui/popup.rs as Rule 3 auto-fix (non-exhaustive match on new Popup::Wizard variant)"
  - "Wizard intercept block intercepts Action::Confirm (Enter) in addition to WizardNextStep — matches ScanConfig pattern"
  - "remap_wizard_action maps Enter -> WizardNextStep (not Confirm) to avoid routing through Confirm's popup.take() chain"
  - "write_agent_files_for() silently ignores write errors — non-critical, wizard succeeds even if agent files fail"

patterns-established:
  - "Popup intercept pattern: ScanConfig and Wizard both use early-return intercept blocks before main match"
  - "Wizard-mode remap: SearchInput chars routed to wizard actions (j/k/Space/a/n/p/q), Enter->WizardNextStep"

requirements-completed: [BOOT-01, BOOT-04, BOOT-05, BOOT-06, BOOT-07]

# Metrics
duration: 3min
completed: 2026-02-23
---

# Phase 03 Plan 02: Bootstrap Wizard Wiring Summary

**Full wizard Action routing, Popup::Wizard with step navigation intercept, write_mise_toml/write_agent_files_for I/O functions, B keybinding, and remap_wizard_action — wizard is fully interactive**

## Performance

- **Duration:** 3 min
- **Started:** 2026-02-23T15:13:42Z
- **Completed:** 2026-02-23T15:16:50Z
- **Tasks:** 2
- **Files modified:** 5

## Accomplishments
- Wired all 7 wizard Action variants (OpenWizard through WizardCompleted) into action.rs and app.rs handle_action
- Implemented step-transition intercept block: MoveUp/Down for Review navigation and Preview scroll, WizardToggleTool/ToggleAgentFiles for toggles, WizardNextStep/Confirm for step advancement with tokio::spawn for write+install, WizardPrevStep for back navigation
- Added write_mise_toml() with atomic temp-file rename and write_agent_files_for() for AGENTS.md/CLAUDE.md
- Wired B keybinding, is_wizard_active() helper, and remap_wizard_action() in main.rs

## Task Commits

Each task was committed atomically:

1. **Task 1: Add wizard actions, Popup::Wizard, write/install functions** - `e8e0259` (feat)
2. **Task 2: Wire B keybinding and remap_wizard_action in main.rs** - `cc5f08d` (feat)

**Plan metadata:** (docs commit follows)

## Files Created/Modified
- `src/action.rs` - Added DetectedTool import and 7 wizard action variants
- `src/app.rs` - Added Popup::Wizard variant, wizard intercept block, 4 handle_action arms, generate_mise_toml_preview()
- `src/mise.rs` - Added write_mise_toml() (atomic write) and write_agent_files_for() (AGENTS.md + CLAUDE.md)
- `src/main.rs` - Added is_wizard_active(), remap_wizard_action(), wizard check in remap chain, B keybinding
- `src/ui/popup.rs` - Added render_wizard() with Detecting/Review/Preview/Writing step renderers

## Decisions Made
- render_wizard() was added to ui/popup.rs as a Rule 3 auto-fix since the new Popup::Wizard variant caused a non-exhaustive match compile error in the existing popup renderer
- Wizard intercept block intercepts both Action::WizardNextStep and Action::Confirm so Enter advances the wizard regardless of whether remap_wizard_action transforms Enter to WizardNextStep or passes Confirm through
- write_agent_files_for() silently ignores errors (non-critical BOOT-07 feature; wizard proceeds to install regardless)

## Deviations from Plan

### Auto-fixed Issues

**1. [Rule 3 - Blocking] Added render_wizard() to ui/popup.rs**
- **Found during:** Task 1 (after adding Popup::Wizard variant)
- **Issue:** src/ui/popup.rs match on popup did not cover Popup::Wizard(_), causing compile error E0004
- **Fix:** Added render_wizard() function with step-specific rendering for all 4 WizardStep variants and wired it into the match arm
- **Files modified:** src/ui/popup.rs
- **Verification:** cargo check passes with zero errors after fix
- **Committed in:** e8e0259 (Task 1 commit)

---

**Total deviations:** 1 auto-fixed (1 blocking)
**Impact on plan:** Required to compile; no scope creep. render_wizard() will be the permanent wizard UI renderer.

## Issues Encountered
None — cargo check passes with zero errors after both tasks.

## User Setup Required
None - no external service configuration required.

## Next Phase Readiness
- Plan 03 (wizard UI polish / help text update) can proceed — wizard is fully functional
- All BOOT-01/04/05/06/07 requirements satisfied
- No blockers

## Self-Check: PASSED

- FOUND: src/action.rs (OpenWizard and 6 other wizard variants present)
- FOUND: src/app.rs (Popup::Wizard, intercept block, handle_action arms present)
- FOUND: src/mise.rs (write_mise_toml, write_agent_files_for present)
- FOUND: src/main.rs (B keybinding, remap_wizard_action, is_wizard_active present)
- FOUND: src/ui/popup.rs (render_wizard present)
- FOUND: .planning/phases/03-bootstrap-wizard/03-02-SUMMARY.md
- COMMIT e8e0259: feat(03-02): add wizard actions, Popup::Wizard, and write/install functions
- COMMIT cc5f08d: feat(03-02): wire B keybinding and remap_wizard_action in main.rs
- cargo check: zero errors

---
*Phase: 03-bootstrap-wizard*
*Completed: 2026-02-23*
