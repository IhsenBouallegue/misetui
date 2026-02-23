# misetui — Feature Roadmap

The goal of these features is to transform misetui from a mise browser into a
**project environment manager** — the tool you open when you sit down to work,
not just when something breaks. Together they cover the full lifecycle: discover
project health, bootstrap new projects, edit requirements in-place, and see
drift the moment it happens.

---

## Feature 1: Projects Tab — Multi-Project Dashboard

### Problem
misetui currently only shows global mise state. Developers work across many
projects, each with different tool requirements. There is no way to see the
health of all your projects at a glance or manage their tool requirements
without `cd`-ing into each one.

### Vision
A new **Projects** tab (inserted between Config and Doctor) that scans
configured directories for `.mise.toml` / `.tool-versions` / `.nvmrc` etc.,
parses their requirements, and shows a live health dashboard.

### UI Layout

```
Projects                                                     [3 / 7 healthy]
──────────────────────────────────────────────────────────────────────────────
  Name               Path                   Tools   Status       Last seen
  ───────────────────────────────────────────────────────────────────────────
● api-service        ~/projects/api          6      ✓ healthy    today
● frontend           ~/projects/frontend     4      ✗ 2 missing  3d ago
● data-pipeline      ~/projects/pipeline     8      ⚠ 1 outdated 1w ago
  cli-tool           ~/projects/cli          3      ✓ healthy    2w ago
```

Status colours:
- `✓ healthy` — all required tools installed at the pinned version
- `⚠ N outdated` — all installed but pins are behind latest
- `✗ N missing` — one or more required tools not installed at all

### Drill-down view (Enter on a project)

Pressing `Enter` opens a detail panel showing the project's requirements vs
installed state:

```
api-service  ~/projects/api-service/.mise.toml
──────────────────────────────────────────────
  Tool       Required     Installed    Status
  ─────────────────────────────────────────
  node       22           22.14.0      ✓
  python     3.12         3.12.7       ✓
  terraform  >= 1.6       —            ✗ missing
  bun        latest       1.1.38       ✓
```

### Keybindings (Projects tab)

| Key | Action |
|-----|--------|
| `Enter` | Drill into project requirements |
| `i` | Install all missing tools for selected project |
| `u` | Update all outdated pins for selected project |
| `e` | Open project's `.mise.toml` in the inline editor (Feature 2) |
| `r` | Re-scan projects |
| `/` | Fuzzy search project names / paths |

### Configuration

Scan directories are read from `~/.config/misetui/config.toml`:

```toml
[projects]
scan_dirs = [
  "~/projects",
  "~/work",
]
max_depth = 2
```

If no config exists, default to `~/projects` and the current working directory.

### Implementation Notes

- Parse config files without shelling out where possible; use a TOML parser for
  `.mise.toml` and read plain text for `.nvmrc` / `.python-version` etc.
- Cross-reference required tools against the already-loaded `InstalledTool` list
  to determine health — no extra `mise` calls for the dashboard view.
- Last-seen is derived from the config file's `mtime`.
- Add `Tab::Projects` to the `Tab` enum between `Config` and `Doctor`.
- Add `ProjectsLoaded(Vec<ProjectStatus>)` action.
- New model structs: `ProjectStatus { name, path, config_path, requirements:
  Vec<ProjectRequirement>, health: ProjectHealth }` and `ProjectRequirement {
  tool, version_spec, installed_version: Option<String> }`.

---

## Feature 2: Interactive `.mise.toml` Editor

### Problem
The Config tab is read-only. Adding a tool to a project requires context-
switching to `$EDITOR`, knowing TOML syntax, and manually formatting the file.
This is the most common mise workflow and it has no TUI support.

### Vision
Press `e` on any config file (from Config tab or Projects tab) to open a
structured inline editor. Changes are written back to disk immediately on
confirm.

### UI Layout

```
Editing: ~/projects/api-service/.mise.toml
──────────────────────────────────────────────────────────────────────────────
  [tools]
  ▶ node       = "22"             [e]dit  [d]elete
    python     = "3.12"           [e]dit  [d]elete
    terraform  = ">= 1.6"         [e]dit  [d]elete

  [env]
  ▶ DATABASE_URL = "postgres://…" [e]dit  [d]elete
    NODE_ENV      = "development" [e]dit  [d]elete

  [tasks]
  ▶ build   = "cargo build"       [e]dit  [d]elete
    test    = "cargo test"        [e]dit  [d]elete

  [a] Add tool   [A] Add env var   [T] Add task   [w] Write & close   [Esc] Cancel
```

### Operations

**Add tool** (`a`):
1. Opens the registry picker (reuses the existing registry + version picker
   popup stack).
2. Optionally type a version spec (`22`, `>= 20`, `latest`).
3. Appends the entry under `[tools]` in the file.

**Edit tool version** (`e` on a tool row):
1. Opens version picker pre-filtered to that tool.
2. Selecting a version rewrites the line in-place.

**Delete tool** (`d` on a tool row):
1. Confirmation dialog.
2. Removes the line from the file.

**Add / edit env var** (`A` / `e` on env row):
1. Two-field inline text entry: key, value.
2. Writes or updates the `[env]` section.

**Add / edit task** (`T` / `e` on task row):
1. Single-line command entry with task name.
2. Writes or updates the `[tasks]` section.

**Write** (`w`): serialise the in-memory representation back to TOML and write
to disk. Show a status message on success.

### Implementation Notes

- Use the `toml_edit` crate to parse and mutate the file while preserving
  formatting and comments.
- Keep an in-memory document (`toml_edit::Document`) as the editor state.
- Add `AppMode::EditingConfig` alongside the existing normal/search/version-
  picker modes and update `remap_*` accordingly.
- On `w`, write to a temp file then `rename` atomically to avoid partial writes.
- Trigger a config + tools refresh after a successful write.

---

## Feature 3: Project Bootstrap Wizard

### Problem
Starting a new mise-managed project requires manually creating `.mise.toml`,
knowing which tools to add, finding the right version, and running `mise
install`. This friction discourages adoption.

### Vision
Press `B` from any tab (or the Projects tab when a project shows no config) to
open a guided wizard that detects your project type, suggests tools, and
generates a ready-to-use `.mise.toml`.

### UI Flow

**Step 1 — Detect & suggest**

```
Bootstrap: ~/projects/new-api
──────────────────────────────────────────────────────────────────────────────
  Detected files:
    package.json        → node (suggested: 22.14.0 LTS)
    .nvmrc              → node 20  (will migrate pin)

  Suggested tools:
  [x] node    22.14.0 LTS    (change version?)
  [x] pnpm    latest
  [ ] bun     1.1.38         (add?)

  [a] Add tool from registry     [Space] toggle     [Enter] Next →
```

**Step 2 — Preview generated config**

```
  Generated .mise.toml:
  ─────────────────────
  [tools]
  node = "22"
  pnpm = "latest"

  [Esc] Back     [w] Write & install
```

**Step 3 — Install**

Immediately runs `mise install` in the target directory and streams progress via
the existing spinner/progress popup.

### Auto-detection Rules

| File present | Suggested tools |
|---|---|
| `package.json` | `node`, `pnpm` or `yarn` if lock file present |
| `Cargo.toml` | `rust` |
| `pyproject.toml` / `requirements.txt` | `python`, `uv` |
| `go.mod` | `go` |
| `Gemfile` | `ruby` |
| `composer.json` | `php` |
| `.nvmrc` | `node` at pinned version |
| `.python-version` | `python` at pinned version |
| `.ruby-version` | `ruby` at pinned version |
| `.tool-versions` | migrate all pins directly |

Migration from legacy files (`nvmrc`, `.python-version`, `.tool-versions`) reads
the exact version and pre-fills it so the pin is preserved, not lost.

### Implementation Notes

- Wizard state machine: `WizardStep::Detect → Review → Preview → Installing`.
- Detection scans the current working directory (or the path of a selected
  project in the Projects tab).
- Version suggestions: reuse `fetch_versions()` to find the latest stable for
  each suggested tool; cache results to avoid redundant network calls.
- After writing `.mise.toml`, run `mise install` in that directory (pass `--cd`
  flag or set `CWD` on the child process).
- Add `Action::OpenBootstrap` bound to `B` in normal mode.

---

## Feature 4: Environment Drift Indicator

### Problem
When you switch projects or pull new changes that update `.mise.toml`, there is
no immediate signal that your active environment is out of sync. You only
discover drift when something fails.

### Vision
A persistent health indicator in the **header bar** showing whether the current
working directory's tool requirements match what is actually installed and
active. Updates live via filesystem watch — no manual refresh needed.

### UI

Header bar (current):
```
 misetui                                        Tools | Outdated | Registry | …
```

Header bar (with drift indicator):
```
 misetui  ~/projects/api  ⚠ 1 tool drifted: node 20 active, 22 required    …
```

States:
- ` ✓ env healthy` — all required tools match installed versions (shown in muted green)
- `⚠ N drifted` — version mismatch between requirement and active version
- `✗ N missing` — required tools not installed at all
- `◌ no config` — no `.mise.toml` found in the cwd (shown dimmed)
- `⟳ checking…` — briefly shown while re-evaluating after a file change

Clicking or pressing `?` on the indicator jumps to the Projects drill-down for
the current project.

### File Watching

Use the `notify` crate to watch:
- The current working directory for `.mise.toml` / `.tool-versions` changes.
- `~/.config/mise/config.toml` (global config) for changes.

On a change event, re-parse the config and re-evaluate drift against the
already-loaded tool list. Emit a `DriftUpdated(DriftStatus)` action into the
existing event loop — no blocking, no extra `mise` subprocess.

### Implementation Notes

- Add a `watcher: Option<notify::RecommendedWatcher>` field to `App`.
- Initialise the watcher on startup; debounce events (200 ms) to avoid thrashing
  on rapid saves.
- `DriftStatus` enum: `Healthy`, `Drifted(Vec<DriftItem>)`, `Missing(Vec<String>)`,
  `NoConfig`, `Checking`.
- Drift evaluation is pure computation against in-memory data — no subprocess.
- Update the header renderer (`src/ui/header.rs`) to render the indicator on the
  right side of the title row.
- CWD is captured once at startup via `std::env::current_dir()`; re-read after
  each file-watch event.

---

## Implementation Order

These features have natural dependencies:

1. **Projects Tab** first — establishes the `ProjectStatus` / `ProjectRequirement`
   models and project-scanning logic that Features 2, 3, and 4 all build on.
2. **Drift Indicator** second — reuses project-scanning and can be developed
   independently of the editor; high visibility payoff with modest scope.
3. **Bootstrap Wizard** third — reuses the project scanner, the version picker,
   and the registry; introduces the wizard state machine.
4. **Inline Editor** last — most complex (TOML round-trip editing, new app mode);
   benefits from the registry picker and version picker already being solid.

---

## New Dependencies

| Crate | Purpose |
|---|---|
| `toml_edit` | Parse and mutate TOML files while preserving formatting |
| `notify` | Cross-platform filesystem watching for drift detection |
| `dirs` | Resolve `~` in configured scan paths |
