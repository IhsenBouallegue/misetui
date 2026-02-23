# Roadmap: misetui — Project Environment Manager

## Overview

This milestone transforms misetui from a read-only mise browser into a full project environment manager. Four features ship in dependency order: a multi-project health dashboard (Projects Tab), a live drift detector in the header (Drift Indicator), a guided wizard for bootstrapping new projects (Bootstrap Wizard), and a structured in-place TOML editor for config files (Inline Editor). Each phase delivers one complete, independently verifiable capability.

## Phases

**Phase Numbering:**
- Integer phases (1, 2, 3): Planned milestone work
- Decimal phases (2.1, 2.2): Urgent insertions (marked with INSERTED)

Decimal phases appear between their surrounding integers in numeric order.

- [ ] **Phase 1: Projects Tab** - Multi-project health dashboard with drill-down and install/update actions
- [x] **Phase 2: Drift Indicator** - Persistent header-bar indicator with live filesystem watch (completed 2026-02-23)
- [ ] **Phase 3: Bootstrap Wizard** - Guided .mise.toml generator with auto-detection and install
- [ ] **Phase 4: Inline Editor** - Structured TOML editor with atomic writes and toml_edit round-trip

## Phase Details

### Phase 1: Projects Tab
**Goal**: Users can see and act on the health of all their mise-managed projects from a single tab
**Depends on**: Nothing (builds on existing app infrastructure)
**Requirements**: PROJ-01, PROJ-02, PROJ-03, PROJ-04, PROJ-05, PROJ-06, PROJ-07
**Success Criteria** (what must be TRUE):
  1. User opens Projects tab and sees a list of all projects found in configured scan directories, each showing name, path, tool count, and health status
  2. User can drill into a project and see per-tool required vs installed version with health status for each tool
  3. User can press `i` on a selected project to install all missing tools, and `u` to update all outdated tool pins
  4. User can fuzzy-search project names/paths with `/` and rescan with `r`
  5. User can configure scan directories and max depth via `~/.config/misetui/config.toml`; defaults apply when no config exists
**Plans**: 4 plans

Plans:
- [ ] 01-01-PLAN.md — Config loader, health model types (MiseProject/ProjectToolHealth/ProjectHealthStatus), scan_projects() in mise.rs
- [ ] 01-02-PLAN.md — Tab::Projects in App state, action wiring (ProjectsLoaded/InstallProjectTools/UpdateProjectPins), start_fetch() integration, i/u/r/search keybindings
- [ ] 01-03-PLAN.md — src/ui/projects.rs renderer: project list with health badges + per-tool drill-down sub-view
- [ ] 01-04-PLAN.md — Human verification of all PROJ-01 through PROJ-07 requirements

### Phase 2: Drift Indicator
**Goal**: Users always know whether their current working directory's tool requirements are satisfied, without manually checking
**Depends on**: Phase 1 (DRFT-03 jumps to Projects drill-down)
**Requirements**: DRFT-01, DRFT-02, DRFT-03
**Success Criteria** (what must be TRUE):
  1. A persistent indicator in the header bar shows healthy / drifted / missing / no-config / checking state for the CWD at all times
  2. When `.mise.toml` or `~/.config/mise/config.toml` changes on disk, the indicator updates automatically within ~200ms — no `r` required
  3. User can press `?` on the drift indicator to jump directly to the Projects drill-down for the current project
**Plans**: 3 plans

Plans:
- [ ] 02-01-PLAN.md — DriftState model, check_cwd_drift() mise function, notify crate dependency
- [ ] 02-02-PLAN.md — App drift_state field, header indicator rendering, Action enum additions, P keybinding (DRFT-01, DRFT-03)
- [ ] 02-03-PLAN.md — Filesystem watcher (notify) spawned at startup, debounced ~200ms auto-refresh (DRFT-02)

### Phase 3: Bootstrap Wizard
**Goal**: Users can generate a correct `.mise.toml` for any project directory in under a minute, without knowing the mise config format
**Depends on**: Phase 1 (wizard can target a selected project with no config)
**Requirements**: BOOT-01, BOOT-02, BOOT-03, BOOT-04, BOOT-05, BOOT-06, BOOT-07
**Success Criteria** (what must be TRUE):
  1. User presses `B` from any tab to open the wizard; wizard detects project type from filesystem files and pre-selects appropriate tools
  2. User can review and toggle the tool list, preview the generated `.mise.toml` content, then confirm — config is written and `mise install` runs with streaming progress
  3. Legacy version pins from `.nvmrc`, `.python-version`, `.ruby-version`, and `.tool-versions` are carried over with exact versions preserved
  4. Optionally, `AGENTS.md` and `CLAUDE.md` agent instruction files are written alongside `.mise.toml`
**Plans**: TBD

### Phase 4: Inline Editor
**Goal**: Users can add, edit, and delete tools, env vars, and tasks in any `.mise.toml` directly from the TUI without touching a text editor
**Depends on**: Phase 1 (editor opens from Projects tab), Phase 2 (refresh triggered after save)
**Requirements**: EDIT-01, EDIT-02, EDIT-03, EDIT-04, EDIT-05, EDIT-06, EDIT-07, EDIT-08, EDIT-09
**Success Criteria** (what must be TRUE):
  1. User presses `e` on any config file in the Config tab or Projects tab to open a structured TOML editor showing tools, env vars, and tasks as editable rows
  2. User can add a tool with `a` (opens registry + version picker), edit a tool version with `e` (opens version picker pre-filtered), and delete a tool with `d` (with confirmation)
  3. User can add and edit `[env]` key/value entries with `A`/`e` and `[tasks]` name/command entries with `T`/`e`
  4. User presses `w` to write changes; file is written atomically (temp file + rename) via `toml_edit`, original formatting and comments are preserved, and a success message is shown
  5. After a successful write, the app refreshes config and tools state automatically
**Plans**: TBD

## Progress

**Execution Order:**
Phases execute in numeric order: 1 → 2 → 3 → 4

| Phase | Plans Complete | Status | Completed |
|-------|----------------|--------|-----------|
| 1. Projects Tab | 3/4 | In Progress|  |
| 2. Drift Indicator | 3/3 | Complete    | 2026-02-23 |
| 3. Bootstrap Wizard | 0/TBD | Not started | - |
| 4. Inline Editor | 0/TBD | Not started | - |
