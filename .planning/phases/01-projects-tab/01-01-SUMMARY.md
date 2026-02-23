---
phase: 01-projects-tab
plan: 01
subsystem: data-foundation
tags: [config, model, scanner, projects-tab]
dependency_graph:
  requires: []
  provides: [MisetuiConfig, MiseProject, ProjectHealthStatus, ProjectToolHealth, scan_projects]
  affects: [src/app.rs, src/ui/]
tech_stack:
  added: [dirs = "5", toml = "0.8"]
  patterns: [worst-case health aggregation, recursive dir walker with depth limit, config-or-default pattern]
key_files:
  created:
    - src/config.rs
  modified:
    - src/model.rs
    - src/mise.rs
    - Cargo.toml
    - src/main.rs
decisions:
  - "scan_projects() is synchronous (filesystem I/O only) to be wrapped in tokio::spawn in plan 02"
  - "Health aggregation: Missing > Outdated > Healthy (worst-case wins)"
  - "collect_projects() stops recursing into dirs that contain .mise.toml (no nested projects)"
  - "Hidden dirs, node_modules, and target are skipped during directory traversal"
metrics:
  duration: "111s"
  completed: "2026-02-23"
  tasks_completed: 3
  tasks_total: 3
  files_created: 1
  files_modified: 4
---

# Phase 01 Plan 01: Data Foundation for Projects Tab Summary

**One-liner:** TOML config loader with dir scanner and worst-case health aggregation for MiseProject discovery.

## What Was Built

A complete data foundation layer for the Projects tab comprising three components:

1. **`src/config.rs`** — `MisetuiConfig` struct with `load()` method that reads `~/.config/misetui/config.toml` via `dirs::config_dir()` and returns defaults (`~/projects` + CWD, depth 3) when the file is absent or unparseable.

2. **`src/model.rs` additions** — Three new types: `ProjectHealthStatus` enum (Healthy/Outdated/Missing/NoConfig) with `label()` method; `ProjectToolHealth` struct for per-tool breakdown rows; `MiseProject` struct holding name, path, tool_count, aggregate health, and tool list.

3. **`src/mise.rs` additions** — `scan_projects()` synchronous function that builds an installed-tool lookup map (name→version for active tools), recursively walks configured scan directories up to `max_depth`, parses each `.mise.toml` for `[tools]` entries, and computes worst-case health without spawning any subprocess. Helper functions `collect_projects()` and `parse_project()` handle the walk and per-file parsing respectively.

## Decisions Made

- `scan_projects()` is synchronous because it performs only filesystem I/O — it will be called inside `tokio::spawn(async move { ... })` in plan 02 to avoid blocking the async runtime.
- Health aggregation uses worst-case: Missing overrides Outdated overrides Healthy.
- Directory traversal stops recursing into a directory once a `.mise.toml` is found there (no nested project scanning).
- Hidden directories (`.git`, etc.), `node_modules`, and `target` are skipped to avoid slow scans.
- Duplicate paths (when the same dir appears in multiple scan roots) are deduplicated before final sort by name.

## Deviations from Plan

None — plan executed exactly as written.

## Self-Check: PASSED

Files created/verified:
- FOUND: /home/ihsen/Documents/repos/misetui/src/config.rs
- FOUND: MisetuiConfig in src/config.rs
- FOUND: MiseProject in src/model.rs
- FOUND: scan_projects() in src/mise.rs
- FOUND: dirs and toml in Cargo.toml

Commits verified:
- 40fc9b9: feat(01-01): add dirs/toml crates and MisetuiConfig loader
- 26643e9: feat(01-01): add MiseProject health model types to model.rs
- ae5a4c8: feat(01-01): implement scan_projects() in mise.rs
