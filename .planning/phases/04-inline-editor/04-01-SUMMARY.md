---
phase: 04-inline-editor
plan: 01
subsystem: ui
tags: [toml_edit, editor, popup, rust, tui]

# Dependency graph
requires:
  - phase: 03-bootstrap-wizard
    provides: "Popup enum pattern, intercept block pattern, wizard flow reference implementation"
provides:
  - "EditorState, EditorTab, EditorToolRow, EditorEnvRow, EditorTaskRow, EditorRowStatus types in model.rs"
  - "parse_config_for_editor() parses .mise.toml into EditorState via toml_edit"
  - "write_editor_changes() applies diffs atomically to .mise.toml preserving comments/formatting"
  - "Action enum: OpenEditor, EditorLoaded, EditorWrite, EditorWriteComplete, and all editor stubs"
  - "Popup::Editor(Box<EditorState>) variant wired into app.rs"
  - "is_editor_active() + remap_editor_action() stub in main.rs"
  - "'e' key on Config/Projects tab dispatches OpenEditor"
affects: [04-02-editor-behavior, 04-03-editor-renderer]

# Tech tracking
tech-stack:
  added: [toml_edit 0.22]
  patterns:
    - "raw_document: String pattern — store serialized toml_edit::DocumentMut as String in EditorState so model.rs stays free of the toml_edit dependency; re-parsed in write_editor_changes()"
    - "Box<EditorState> in Action/Popup — large struct boxed to avoid Action enum size bloat"
    - "EditorStartEdit dispatched by 'e' key, handles context-sensitive path resolution (Config tab vs Projects tab)"

key-files:
  created: []
  modified:
    - Cargo.toml
    - src/model.rs
    - src/mise.rs
    - src/action.rs
    - src/app.rs
    - src/main.rs
    - src/ui/popup.rs

key-decisions:
  - "raw_document stored as String in EditorState so model.rs has zero dependency on toml_edit; mise.rs re-parses it during write_editor_changes()"
  - "EditorLoaded wraps EditorState in Box to avoid Action enum size bloat"
  - "EditorStartEdit dispatched by 'e' key in remap_normal_action; context-sensitive path resolution in handle_action (Config tab path vs Projects tab .mise.toml join)"
  - "Placeholder Popup::Editor render added to popup.rs to avoid non-exhaustive match compile error; full renderer deferred to Plan 03"

patterns-established:
  - "Editor intercept block pattern: is_editor_active() guard + remap_editor_action() stub, mirroring is_wizard_active/remap_wizard_action pattern from Phase 03"

requirements-completed: [EDIT-01, EDIT-07, EDIT-08]

# Metrics
duration: 2min
completed: 2026-02-23
---

# Phase 4 Plan 01: Inline Editor Data Foundation Summary

**toml_edit round-trip TOML editor foundation: EditorState types, parse_config_for_editor/write_editor_changes in mise.rs, Popup::Editor variant, OpenEditor/'e' key entry point**

## Performance

- **Duration:** 2 min
- **Started:** 2026-02-23T13:22:29Z
- **Completed:** 2026-02-23T13:24:49Z
- **Tasks:** 2
- **Files modified:** 7

## Accomplishments

- Added toml_edit 0.22 dependency and full EditorState/EditorTab/EditorRowStatus/EditorToolRow/EditorEnvRow/EditorTaskRow types to model.rs
- Implemented parse_config_for_editor() parsing [tools]/[env]/[tasks] tables via toml_edit::DocumentMut into EditorState, storing raw_document as String for round-trip preservation
- Implemented write_editor_changes() applying Modified/Added/Deleted row changes atomically to the Document then temp-file+rename write
- Wired full Action enum editor variants, Popup::Editor(Box<EditorState>), OpenEditor/EditorLoaded/EditorStartEdit/EditorWriteComplete handle_action arms, and is_editor_active()/remap_editor_action() stub in main.rs with 'e' keybinding

## Task Commits

Each task was committed atomically:

1. **Task 1: Add toml_edit dependency and editor model types** - `ae5bd9d` (feat)
2. **Task 2: Add parse/write functions and editor Action/Popup wiring** - `42322ca` (feat)

## Files Created/Modified

- `Cargo.toml` - Added toml_edit = "0.22" dependency
- `src/model.rs` - Added EditorTab, EditorRowStatus, EditorToolRow, EditorEnvRow, EditorTaskRow, EditorState types
- `src/mise.rs` - Added parse_config_for_editor() and write_editor_changes() functions with toml_edit imports
- `src/action.rs` - Added EditorState import and 14 editor Action variants (OpenEditor through EditorBackspace)
- `src/app.rs` - Added EditorState import, Popup::Editor variant, and OpenEditor/EditorLoaded/EditorStartEdit/EditorWriteComplete handle_action arms
- `src/main.rs` - Added 'e' -> EditorStartEdit keybinding, is_editor_active(), remap_editor_action() stub, event loop check
- `src/ui/popup.rs` - Added placeholder Popup::Editor match arm rendering "Loading editor..." text

## Decisions Made

- raw_document stored as String in EditorState so model.rs has zero dependency on toml_edit; mise.rs re-parses it during write_editor_changes(). This keeps model.rs dependency-free while enabling full round-trip TOML preservation.
- EditorLoaded wraps EditorState in Box to avoid bloating Action enum size (EditorState is a large struct with multiple Vecs).
- EditorStartEdit dispatched by 'e' key handles context-sensitive path resolution in handle_action — Config tab uses visible_configs_vec()[config_selected].path, Projects tab joins .mise.toml onto the project path.
- Non-exhaustive Popup::Editor match in Action::Confirm handler fixed as Rule 3 auto-fix (same pattern as ScanConfig arm).

## Deviations from Plan

### Auto-fixed Issues

**1. [Rule 3 - Blocking] Added Popup::Editor arm to Action::Confirm match in app.rs**
- **Found during:** Task 2 (editor Action/Popup wiring)
- **Issue:** The Action::Confirm handler has a `match popup { ... }` that was non-exhaustive after adding Popup::Editor — cargo check error E0004
- **Fix:** Added `Popup::Editor(_) => {}` arm with comment "Editor confirm is handled by the intercept block (Plan 02)"
- **Files modified:** src/app.rs
- **Verification:** cargo check passes with zero errors
- **Committed in:** 42322ca (Task 2 commit)

---

**Total deviations:** 1 auto-fixed (1 blocking)
**Impact on plan:** Required fix for code to compile — no scope creep.

## Issues Encountered

None beyond the Popup::Editor non-exhaustive match auto-fix above.

## Next Phase Readiness

- All data foundation types and I/O plumbing are in place for Plan 02 (editor behavior: intercept block, key routing, row editing) and Plan 03 (editor.rs renderer)
- 'e' key on Config tab opens a Progress popup then dispatches EditorLoaded → Popup::Editor
- parse_config_for_editor() and write_editor_changes() are fully implemented and tested at compile time
- No blockers

---
*Phase: 04-inline-editor*
*Completed: 2026-02-23*
