# Requirements: misetui — Project Environment Manager

**Defined:** 2026-02-23
**Core Value:** See all your project environments at a glance and act on them without leaving the terminal.

## v1 Requirements

### Projects Tab

- [ ] **PROJ-01**: User can see a health dashboard listing all projects found in configured scan directories, showing name, path, tool count, health status, and last-seen date
- [ ] **PROJ-02**: User can configure scan directories and max depth via `~/.config/misetui/config.toml`; default to `~/projects` and CWD when no config exists
- [ ] **PROJ-03**: User can drill into a selected project to see a per-tool breakdown of required version vs installed version and health status
- [ ] **PROJ-04**: User can install all missing tools for a selected project with `i`
- [ ] **PROJ-05**: User can update all outdated tool pins for a selected project with `u`
- [ ] **PROJ-06**: User can fuzzy-search project names and paths with `/`
- [ ] **PROJ-07**: User can rescan projects with `r`

### Drift Indicator

- [x] **DRFT-01**: User sees a persistent health indicator in the header bar showing whether the CWD's tool requirements match what is installed (healthy / drifted / missing / no-config / checking states)
- [x] **DRFT-02**: Indicator updates live when `.mise.toml` or `~/.config/mise/config.toml` changes on disk — no manual refresh required
- [ ] **DRFT-03**: User can press `?` on the drift indicator to jump to the Projects drill-down for the current project

### Bootstrap Wizard

- [ ] **BOOT-01**: User can open a bootstrap wizard with `B` from any tab to generate a `.mise.toml` for the current working directory (or a selected project with no config)
- [ ] **BOOT-02**: Wizard auto-detects project type from filesystem files (package.json → node/pnpm, Cargo.toml → rust, pyproject.toml/requirements.txt → python/uv, go.mod → go, Gemfile → ruby, composer.json → php)
- [ ] **BOOT-03**: Wizard migrates pins from legacy files (`.nvmrc`, `.python-version`, `.ruby-version`, `.tool-versions`) preserving exact versions
- [ ] **BOOT-04**: User can toggle/add tools in the Review step before writing the config
- [ ] **BOOT-05**: User can preview the generated `.mise.toml` content before writing
- [ ] **BOOT-06**: Wizard writes `.mise.toml` and immediately runs `mise install` in the target directory, streaming progress via the existing popup
- [ ] **BOOT-07**: Wizard optionally writes AI agent instruction files (`AGENTS.md` and `CLAUDE.md`) alongside `.mise.toml` containing mise-specific guidance (how to run tasks, install tools, and respect pinned versions) compatible with Claude Code, Cursor, and generic AGENTS.md conventions

### Inline Editor

- [ ] **EDIT-01**: User can press `e` on any config file (from Config tab or Projects tab) to open a structured inline TOML editor
- [ ] **EDIT-02**: User can add a tool entry by pressing `a`, which opens the existing registry + version picker flow and appends to `[tools]`
- [ ] **EDIT-03**: User can edit an existing tool's version with `e` on a tool row, which opens the version picker pre-filtered to that tool
- [ ] **EDIT-04**: User can delete a tool entry with `d` on a tool row (with confirmation dialog)
- [ ] **EDIT-05**: User can add and edit `[env]` entries (key/value) with `A` / `e`
- [ ] **EDIT-06**: User can add and edit `[tasks]` entries (name/command) with `T` / `e`
- [ ] **EDIT-07**: User can write changes to disk with `w`; file is written atomically (temp file + rename) and a success message is shown
- [ ] **EDIT-08**: Changes are written using `toml_edit` to preserve original formatting and comments
- [ ] **EDIT-09**: After a successful write, the app triggers a config + tools refresh

## v2 Requirements

### Projects Tab

- **PROJ-V2-01**: Multi-select batch install/update across multiple projects simultaneously
- **PROJ-V2-02**: Project grouping / tagging for organizing large project lists

### Drift Indicator

- **DRFT-V2-01**: Notification popup when drift is detected after a background config change

### Bootstrap Wizard

- **BOOT-V2-01**: Wizard suggests additional tools based on detected framework (e.g. pnpm for Node projects with a pnpm-lock.yaml)

### Inline Editor

- **EDIT-V2-01**: Multi-file editing (open multiple config files in sequence)
- **EDIT-V2-02**: Diff preview showing changes before writing

## Out of Scope

| Feature | Reason |
|---------|--------|
| GUI / web interface | Terminal-first tool by design |
| mise plugin management | Separate concern, not related to project environment management |
| Real-time collaboration | Single-user TUI tool |
| Cloud sync of project configs | Out of scope for a local tool |
| mise shims / PATH management | Core mise CLI concern, not TUI |

## Traceability

| Requirement | Phase | Status |
|-------------|-------|--------|
| PROJ-01 | Phase 1 | Pending |
| PROJ-02 | Phase 1 | Pending |
| PROJ-03 | Phase 1 | Pending |
| PROJ-04 | Phase 1 | Pending |
| PROJ-05 | Phase 1 | Pending |
| PROJ-06 | Phase 1 | Pending |
| PROJ-07 | Phase 1 | Pending |
| DRFT-01 | Phase 2 | Complete |
| DRFT-02 | Phase 2 | Complete |
| DRFT-03 | Phase 2 | Pending |
| BOOT-01 | Phase 3 | Pending |
| BOOT-02 | Phase 3 | Pending |
| BOOT-03 | Phase 3 | Pending |
| BOOT-04 | Phase 3 | Pending |
| BOOT-05 | Phase 3 | Pending |
| BOOT-06 | Phase 3 | Pending |
| BOOT-07 | Phase 3 | Pending |
| EDIT-01 | Phase 4 | Pending |
| EDIT-02 | Phase 4 | Pending |
| EDIT-03 | Phase 4 | Pending |
| EDIT-04 | Phase 4 | Pending |
| EDIT-05 | Phase 4 | Pending |
| EDIT-06 | Phase 4 | Pending |
| EDIT-07 | Phase 4 | Pending |
| EDIT-08 | Phase 4 | Pending |
| EDIT-09 | Phase 4 | Pending |

**Coverage:**
- v1 requirements: 26 total
- Mapped to phases: 26
- Unmapped: 0

---
*Requirements defined: 2026-02-23*
*Last updated: 2026-02-23 after roadmap creation*
