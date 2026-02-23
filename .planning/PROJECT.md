# misetui — Project Environment Manager Milestone

## What This Is

misetui is a Ratatui TUI for the `mise` dev tool version manager. This milestone transforms it from a read-only mise browser into a **project environment manager** — the tool you open when you sit down to work, not just when something breaks. The four new features cover the full project lifecycle: discover health across all projects, detect environment drift live, bootstrap new projects, and edit `.mise.toml` in-place.

## Core Value

See all your project environments at a glance and act on them without leaving the terminal.

## Requirements

### Validated

- ✓ Installed tools browser with fuzzy search — existing
- ✓ Outdated tools view with upgrade actions — existing
- ✓ Registry browser with version picker and install — existing
- ✓ Tasks tab (list and run mise tasks) — existing
- ✓ Environment variables tab — existing
- ✓ Settings tab — existing
- ✓ Config tab (read-only config file viewer) — existing
- ✓ Doctor tab — existing
- ✓ Three-mode input (normal / search / version-picker) — existing
- ✓ Fuzzy filtering with precomputed highlight indices — existing
- ✓ Async action dispatch via MPSC channel — existing
- ✓ Popup-based confirmation and progress dialogs — existing

### Active

- [ ] Projects Tab: scan configured directories for mise config files and show a live health dashboard
- [ ] Projects Tab: drill-down view showing per-tool requirement vs installed state for a selected project
- [ ] Projects Tab: install missing / update outdated tools for a selected project from the tab
- [ ] Environment Drift Indicator: persistent header-bar indicator showing whether the CWD's tool requirements match what is installed
- [ ] Environment Drift Indicator: filesystem watch (notify crate) updates the indicator live on config file changes without manual refresh
- [ ] Bootstrap Wizard: detect project type from filesystem files (package.json, Cargo.toml, etc.) and suggest appropriate tools
- [ ] Bootstrap Wizard: guided wizard UI (Detect → Review → Preview → Installing) that generates and writes `.mise.toml`
- [ ] Bootstrap Wizard: run `mise install` in the target directory after writing the config
- [ ] Inline Editor: press `e` on any config file to open a structured TOML editor inline
- [ ] Inline Editor: add/edit/delete tools, env vars, and tasks with TOML round-trip via `toml_edit` (preserving formatting and comments)
- [ ] Inline Editor: atomic write (temp file + rename) and refresh after successful save

### Out of Scope

- Real-time collaboration or shared config editing — single-user TUI tool
- GUI / web interface — terminal-first by design
- mise plugin management — separate concern, not in this milestone
- Multi-select batch operations across projects — deferred to a future milestone

## Context

Existing codebase is Rust 2021 edition with Ratatui 0.30, Crossterm 0.28, Tokio 1 (full), fuzzy-matcher 0.3, serde/serde_json. Architecture is event-driven: MPSC channel for async actions, three remap functions for input modes, one renderer module per tab, centralized App struct as state.

Three new crates required:
- `toml_edit` — parse and mutate TOML preserving formatting (Inline Editor)
- `notify` — cross-platform filesystem watching (Drift Indicator)
- `dirs` — resolve `~` in configured scan paths (Projects Tab)

The `Tab` enum currently has 8 variants. Projects will be inserted between Config and Doctor (index 8, shifting Doctor to 9).

## Constraints

- **Tech Stack**: Rust / Ratatui / Tokio — no runtime changes
- **mise CLI**: All tool data comes from shelling out to `mise`; parse config files directly (TOML parser) to avoid extra subprocess calls for the health dashboard
- **Implementation Order**: Projects Tab → Drift Indicator → Bootstrap Wizard → Inline Editor (each feature builds on the previous)
- **Atomic Writes**: All config file writes must use temp file + rename to prevent partial-write corruption

## Key Decisions

| Decision | Rationale | Outcome |
|----------|-----------|---------|
| Use `toml_edit` for inline editor | Preserves comments and formatting; standard choice for TOML mutation | — Pending |
| Use `notify` crate for file watching | Cross-platform, integrates cleanly into tokio event loop | — Pending |
| Cross-reference health against in-memory InstalledTool list | Avoids extra `mise` subprocesses for dashboard rendering | — Pending |
| Projects Tab inserted between Config and Doctor | Natural placement; completes the "manage" section of the UI | — Pending |
| Debounce filesystem events at 200ms | Avoids thrashing on rapid saves | — Pending |

---
*Last updated: 2026-02-23 after initialization*
