# Mise CLI Drift Detection Research

**Researched:** 2026-02-23
**Domain:** mise CLI — tool listing, config detection, drift/version-mismatch detection
**Confidence:** HIGH (all findings verified by direct local execution)

---

## Critical Finding: `mise status` is WRONG for drift detection

The existing Plan 02-01 (Task 2) calls `mise status` to detect tool drift. **This is incorrect.**

`mise status` is a **task runner** subcommand. It reports whether _mise tasks_ (defined in `[tasks]` in mise.toml) have run, not whether tools are installed or version-correct. Running `mise status` in a directory without tasks defined produces:

```
mise ERROR no tasks defined in ~/Documents/repos/misetui. Are you in a project directory?
```

Exit code is still 0. It provides zero signal about tool drift.

**The correct command is `mise ls --current --json`.**

---

## Detection Strategy Overview

Three separate detection needs, each with a distinct command:

| Detection Goal | Command | Signal |
|---|---|---|
| Is there a local mise config for CWD? | `mise config ls --json --cd <DIR>` | Check if any `path` entry is not the global config |
| Are required tools missing (not installed)? | `mise ls --current --json --cd <DIR>` | `installed == false` on any entry |
| Version mismatch (config wants X, X is not installed)? | `mise ls --current --json --cd <DIR>` | Same as missing: `installed == false` |
| Untrusted config present? | stderr of any `mise` command contains "not trusted"; exit code 1 | Handle as special error state |

In practice, **a single call to `mise ls --current --json --cd <dir>`** covers all three tool-state cases. The config presence check requires a separate `mise config ls --json` call.

---

## Command 1: `mise ls --current --json --cd <DIR>`

### Purpose
Lists all tools currently active (specified in any config applying to `<DIR>`). Includes both locally-scoped and globally-scoped tools.

### Flags
- `--current` / `-c`: Only show tools specified in a mise.toml (active config). Omits tools installed but not in any active config file.
- `--json` / `-J`: JSON output (uppercase J, note: help also shows `-J`).
- `--cd <DIR>`: Run as if in `<DIR>`. The shell's cwd is irrelevant; mise evaluates config files for `<DIR>`.

### JSON Schema (verified by direct execution)

```json
{
  "<tool-name>": [
    {
      "version": "25.2.1",
      "requested_version": "25",
      "install_path": "/home/user/.local/share/mise/installs/node/25.2.1",
      "source": {
        "type": "mise.toml",
        "path": "/path/to/.mise.toml"
      },
      "installed": true,
      "active": true
    }
  ]
}
```

### Field semantics (verified)

| Field | Type | Meaning |
|---|---|---|
| `version` | string | Resolved concrete version (e.g. `"25.2.1"` when requested `"25"`) |
| `requested_version` | string | Raw spec from config (e.g. `"25"`, `"latest"`, `"1.3.6"`) |
| `install_path` | string | Absolute path to install dir (may not exist if `installed: false`) |
| `source` | object or absent | Config file that activates this tool; absent for orphaned installs |
| `source.type` | string | `"mise.toml"` or `".tool-versions"` etc. |
| `source.path` | string | Absolute path to the config file |
| `installed` | bool | Whether the resolved version is actually on disk |
| `active` | bool | Whether this version is currently active in the resolved env |
| `symlinked_to` | string | (Optional) For tools that use symlinks (e.g. rust) |

### Key behaviors (verified by local experiments)

**Fuzzy/prefix version matching:** When config says `node = "25"` and `25.2.1` is installed, `installed = true` and `version = "25.2.1"`. mise resolves the fuzzy spec to the best-matching installed version.

**`latest` matching:** When config says `rust = "latest"` and latest is installed, `installed = true`.

**Exact version not installed:** When config says `node = "25.1.0"` and only `25.2.1` is installed (different patch), `installed = false`, `active = false`. There is no partial-match fallback for exact version specs.

**Missing = not installed:** `installed: false` is the universal signal for "tool required by config but not present on disk." There is no separate JSON field for "missing." The `--missing` flag in table output mode is cosmetic only; in `--json` mode `installed: false` is the equivalent.

**Exit codes:**
- Exit 0: Command ran successfully (even if some tools have `installed: false`)
- Exit 1: Config file exists but is not trusted (stderr contains "not trusted")
- Exit 1: Other mise error (bad TOML, network failure during check, etc.)

**Untrusted config:** If `<DIR>` contains an untrusted `.mise.toml`, the command writes to stderr and exits 1 with empty stdout. No JSON is produced. stderr example:
```
mise ERROR Config files in /path/.mise.toml are not trusted.
Trust them with `mise trust`.
```

---

## Command 2: `mise config ls --json --cd <DIR>`

### Purpose
Lists all config files that apply to `<DIR>`, in order of precedence (most local first).

### JSON Schema (verified)

```json
[
  {
    "path": "/path/to/project/.mise.toml",
    "tools": ["bun", "java"]
  },
  {
    "path": "/home/user/.config/mise/config.toml",
    "tools": ["node", "rust"]
  }
]
```

### Distinguishing local config from global config

The global config path is always `$XDG_CONFIG_HOME/mise/config.toml` (typically `~/.config/mise/config.toml`). A local project config has a path that starts with or is an ancestor of the project directory.

**Algorithm to detect "has local config":**
```rust
let global_config = dirs::config_dir()
    .map(|d| d.join("mise/config.toml"));

let has_local_config = config_entries.iter().any(|entry| {
    global_config.as_ref()
        .map(|g| entry.path != *g)
        .unwrap_or(true)
});
```

Alternatively, check if any config path is a child of the user's home directory but NOT in `~/.config/mise/`:
```rust
let has_local_config = config_entries.iter().any(|entry| {
    !entry.path.starts_with(&global_config_dir)
});
```

**When there is NO local config:** `mise config ls --json --cd /some/dir-without-mise-toml` returns only the global config entry (or an empty array if no global config either).

**Untrusted config:** Same as `mise ls` — exits 1, empty stdout, error in stderr.

---

## Recommended Implementation: `check_cwd_drift()`

### Algorithm (single-pass)

```rust
pub async fn check_cwd_drift(dir: &Path) -> Result<DriftState, String> {
    // Step 1: Check config ls to detect NoConfig state
    let config_output = Command::new("mise")
        .args(["config", "ls", "--json"])
        .current_dir(dir)
        .output()
        .await
        .map_err(|e| format!("mise not found: {e}"))?;

    // Untrusted config — special case
    let stderr = String::from_utf8_lossy(&config_output.stderr);
    if !config_output.status.success() {
        if stderr.contains("not trusted") {
            return Ok(DriftState::Untrusted); // or map to a suitable variant
        }
        return Err(stderr.to_string());
    }

    let configs: Vec<ConfigEntry> = serde_json::from_slice(&config_output.stdout)
        .map_err(|e| format!("JSON parse error: {e}"))?;

    // Detect if any non-global config applies
    let global_config_path = dirs::config_dir()
        .map(|d| d.join("mise/config.toml"));
    let has_local_config = configs.iter().any(|c| {
        global_config_path.as_ref().map(|g| c.path != *g).unwrap_or(true)
    });

    if !has_local_config {
        return Ok(DriftState::NoConfig);
    }

    // Step 2: Check tool installation state
    let ls_output = Command::new("mise")
        .args(["ls", "--current", "--json"])
        .current_dir(dir)
        .output()
        .await
        .map_err(|e| format!("mise ls failed: {e}"))?;

    if !ls_output.status.success() {
        let stderr = String::from_utf8_lossy(&ls_output.stderr);
        if stderr.contains("not trusted") {
            return Ok(DriftState::Untrusted);
        }
        return Err(stderr.to_string());
    }

    // Parse tool map: { "tool-name": [ { installed: bool, source: {...} } ] }
    let tools: HashMap<String, Vec<ToolEntry>> =
        serde_json::from_slice(&ls_output.stdout)
            .map_err(|e| format!("JSON parse error: {e}"))?;

    // Check only tools that come from a local (non-global) source
    let local_config_paths: HashSet<PathBuf> = configs.iter()
        .filter(|c| global_config_path.as_ref().map(|g| c.path != *g).unwrap_or(true))
        .map(|c| c.path.clone())
        .collect();

    let mut any_missing = false;
    for (_tool, versions) in &tools {
        for v in versions {
            // Only care about tools sourced from local config
            if let Some(source) = &v.source {
                if local_config_paths.contains(&source.path) && !v.installed {
                    any_missing = true;
                }
            }
        }
    }

    if any_missing {
        Ok(DriftState::Missing)
    } else {
        Ok(DriftState::Healthy)
    }
}
```

### Why not `--cd` flag vs `current_dir()`?

Both `--cd <DIR>` and `.current_dir(dir)` on the `Command` builder work. The `--cd` flag is a mise-level flag available on all subcommands; `.current_dir()` sets the process cwd. Either is correct. Using `.current_dir()` is idiomatic Rust and does not require constructing the path as a CLI argument string.

### Why only check local-config-sourced tools?

When a CWD has no local `.mise.toml`, global tools still appear in `mise ls --current --json`. If we flagged globally-configured tools as "drifted" from the CWD perspective, every directory without a local config would appear as needing attention. The TUI feature is about project-level drift, so we scope to local config sources only.

---

## Gotchas and Caveats

### 1. `mise status` is a task command, NOT a tool command (CRITICAL)

`mise status` is documented under task runner functionality. It reports task execution status (sources/outputs freshness), not tool installation state. Using it for drift detection produces wrong results — it will report "no tasks defined" for most projects and exit 0 regardless of tool state.

### 2. `--json` flag is uppercase `-J`

The JSON flag is `-J` (uppercase). Lowercase `-j` sets `--jobs` (parallelism). Both `--json` and `-J` work in long form.

### 3. `installed: false` is the only missing-tool signal

There is no separate `"status": "missing"` field. The only way to detect a missing tool is `installed == false`. The `--missing` flag in non-JSON mode only affects which rows are displayed in the table; it does not add any field to the JSON schema.

### 4. Version mismatch vs missing tool: same signal in JSON

When config requests `node = "25.1.0"` but only `25.2.1` is installed (non-matching exact version), `installed: false`. Mise does not install the "closest" version — it resolves the spec to an exact version and checks if exactly that version is installed.

However, when config uses a fuzzy spec (`node = "25"`) and a matching version is installed (`25.2.1`), `installed: true`. Fuzzy specs are resolved to whatever best-matching version is already installed.

This means "version mismatch" in the sense of "config wants a different minor/patch than what's installed" **only matters for exact version pins.** For fuzzy specs, mise's resolution handles it and reports `installed: true`.

For the TUI's purposes, `installed: false` covers both "not installed at all" and "pinned to a version that isn't installed." Both should map to `DriftState::Missing`.

### 5. Untrusted config = exit 1 with empty stdout

Any `mise` command that encounters an untrusted config exits 1 and writes only to stderr. The stdout is empty — no JSON at all. Always check `output.status.success()` before calling `serde_json::from_slice`.

### 6. `--cd` applies to the mise config resolution, not file I/O

When using `--cd <DIR>` or `.current_dir(dir)`, mise walks up the directory tree from `<DIR>` to find applicable `.mise.toml` files. This is equivalent to the user `cd`-ing into that directory in their shell. The cwd of the Rust process calling `Command` does not matter if you use `--cd`.

### 7. `mise ls --current` includes GLOBAL config tools

`--current` means "tools specified in any active config" — including the global `~/.config/mise/config.toml`. If you only want to report on project-local tools, you must filter by `source.path` after parsing the JSON.

### 8. Large `mise ls` output performance

For directories with many tools, `mise ls --json` includes all installed versions of all tools. `--current` limits to only actively-configured tools, which is significantly smaller. Always use `--current` for drift detection.

---

## Corrected `DriftState` Variants

Based on the research, the enum should be:

```rust
pub enum DriftState {
    Checking,      // async check in flight
    Healthy,       // all local-config tools installed and version-matched
    Missing,       // one or more local-config tools not installed (covers exact-version mismatch too)
    NoConfig,      // no local .mise.toml applies to this directory
    Untrusted,     // local .mise.toml exists but is not trusted by mise
}
```

`Drifted` (version mismatch with tool still installed) is **not detectable** from `mise ls --current --json` alone — `installed: false` does not distinguish between "never installed" and "wrong version installed." Both map to `Missing` since the effect is the same: `mise install` is required.

If distinguishing "never installed" from "wrong version installed" matters for UX, a two-pass strategy could be used: check if the tool name appears in `mise ls --json` (without `--current`) with `installed: true` at any version. But for the drift indicator feature, this level of granularity is likely unnecessary.

---

## Sources

- `mise ls --help` — direct CLI output, current version `2026.1.7 linux-x64 (2026-01-26)`
- `mise ls --json --cd <dir>` — direct execution on multiple test directories
- `mise ls --current --json --cd <dir>` — verified on projects with trusted and untrusted configs
- `mise config ls --json --cd <dir>` — verified on multiple directories
- `mise doctor --json` — verified, not useful for per-directory drift (shows global state only)
- `mise status` — verified to be a task command, irrelevant to tool drift
- https://mise.jdx.dev/cli/ls.html — confirmed field names and flag descriptions
- https://mise.jdx.dev/cli/ — confirmed command list and descriptions
