---
phase: 04-inline-editor
plan: 02
subsystem: ui
tags: [editor, popup, keybinding, intercept, rust, tui]

# Dependency graph
requires:
  - phase: 04-inline-editor
    plan: 01
    provides: "EditorState types, Popup::Editor variant, parse/write functions, remap_editor_action stub"
provides:
  - "remap_editor_action() — full normal-mode key routing for editor popup"
  - "remap_editor_edit_action() — edit-mode key routing (printable chars -> EditorInput, Enter/Esc)"
  - "is_editor_editing() helper splits event routing by editor mode"
  - "Editor intercept block in handle_action handling all 15 EditorXxx action variants"
  - "ConfirmAction::DiscardEditor — unsaved-changes guard when closing with dirty state"
  - "Auto-advance from name column (0) to value column (1) on newly Added rows"
affects: [04-03-editor-renderer]

# Tech tracking
tech-stack:
  added: []
  patterns:
    - "Dual-remap pattern: is_editor_editing() selects between remap_editor_action (nav) and remap_editor_edit_action (typing) — mirrors is_wizard_active/remap_wizard_action precedent"
    - "Editor intercept block: placed BEFORE ScanConfig intercept block, returns early for all editor actions — same pattern as ScanConfig/Wizard intercepts"
    - "Auto-column-advance: EditorConfirmEdit auto-starts editing column 1 when col==0 and row.status==Added — allows seamless name->value entry"

key-files:
  created: []
  modified:
    - src/main.rs
    - src/app.rs

key-decisions:
  - "remap_editor_action maps both h/l to EditorSwitchTab (no direction distinction) — handle_action always cycles forward; users can tab through Tools->Env->Tasks->Tools"
  - "EditorAddTool with 'a' is context-sensitive in handle_action: on Tools tab adds tool row, on Env tab adds env var, on Tasks tab adds task (avoids needing remap to know sub-tab)"
  - "EditorAddEnvVar ('A') and EditorAddTask ('T') force-switch to the appropriate sub-tab so they work from any tab"
  - "ConfirmAction::DiscardEditor added as new variant — DiscardEditor arm in Action::Confirm just falls through (popup already taken by .take())"

patterns-established:
  - "Dual remap functions (normal + edit mode) for modal editor popup — is_editor_editing() guard selects between them in the event loop"

requirements-completed: [EDIT-02, EDIT-03, EDIT-04, EDIT-05, EDIT-06, EDIT-09]

# Metrics
duration: 2min
completed: 2026-02-23
---

# Phase 4 Plan 02: Editor Behavior Implementation Summary

**Full editor action system: remap_editor_action/remap_editor_edit_action key routing, intercept block with all 15 EditorXxx actions, ConfirmAction::DiscardEditor unsaved-changes guard, auto-column-advance for new rows**

## Performance

- **Duration:** 2 min
- **Started:** 2026-02-23T21:47:36Z
- **Completed:** 2026-02-23T21:49:49Z
- **Tasks:** 1
- **Files modified:** 2

## Accomplishments

- Replaced stub `remap_editor_action` with full implementation routing j/k/h/l/e/a/A/T/d/w/q/Esc and Tab/PrevTab/Enter/Esc to correct Action variants in normal navigation mode
- Added `remap_editor_edit_action` routing printable chars to EditorInput, Backspace to EditorBackspace, Enter to EditorConfirmEdit, Esc to EditorCancelEdit
- Added `is_editor_editing()` helper and updated event loop to call the appropriate remap function based on editor mode
- Added editor intercept block in handle_action handling all editor actions: MoveUp/Down/PageUp/PageDown, EditorSwitchTab, EditorStartEdit, EditorConfirmEdit (with auto-column-advance for new rows), EditorCancelEdit, EditorInput, EditorBackspace, EditorDeleteRow, EditorAddTool/AddEnvVar/AddTask, EditorWrite (async write + OperationFailed), EditorClose (dirty guard)
- Added `ConfirmAction::DiscardEditor` variant and handler in Action::Confirm match

## Task Commits

Each task was committed atomically:

1. **Task 1: Implement remap_editor_action and editor intercept block** - `291e881` (feat)

## Files Created/Modified

- `src/main.rs` - Added is_editor_editing(), remap_editor_action (full), remap_editor_edit_action (new), updated event loop for dual-mode routing
- `src/app.rs` - Added ConfirmAction::DiscardEditor, EditorTab/EditorRowStatus/EditorToolRow/EditorEnvRow/EditorTaskRow imports, full editor intercept block (370 lines), DiscardEditor arm in Action::Confirm handler

## Decisions Made

- remap_editor_action maps both h and l to EditorSwitchTab — the intercept block always cycles forward (Tools->Env->Tasks->Tools). Backward cycling would require a separate EditorSwitchTabBack action; kept simple for now.
- EditorAddTool ('a') is context-sensitive: on Tools tab adds tool row; on Env/Tasks tabs adds the appropriate type. This avoids needing the remap function to know the current sub-tab.
- EditorAddEnvVar ('A') and EditorAddTask ('T') explicitly switch to the appropriate sub-tab before entering edit mode — the user can add env vars or tasks from any sub-tab.
- ConfirmAction::DiscardEditor is a new variant — when the user confirms discard, the popup was already taken by `.take()`, so the arm just falls through with no additional action (popup is already None).

## Deviations from Plan

None — plan executed exactly as written.

## Issues Encountered

None.

## Next Phase Readiness

- All editor action handling is fully implemented
- remap_editor_action routes all keybindings correctly
- Editor intercept block handles all 15+ editor actions with proper state mutations
- EditorWrite spawns async write_editor_changes and dispatches EditorWriteComplete which calls start_fetch() (EDIT-09)
- EditorClose with dirty flag shows "Unsaved changes. Discard?" confirm dialog
- Plan 03 (editor renderer) has all the state it needs to render the editor popup

---
*Phase: 04-inline-editor*
*Completed: 2026-02-23*
