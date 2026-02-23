# misetui

A terminal user interface for [mise](https://mise.jdx.dev) — manage your dev tools, versions, tasks, and environment without leaving the terminal.

Built with [Ratatui](https://ratatui.rs) and async Rust.

## Features

### Tabs

- **Tools** — View all installed tools with version, active status, and source. Outdated tools show an inline `current → latest` upgrade arrow. Press `Enter` to inspect tool details.
- **Outdated** — See every tool that has a newer version available, with current, latest, and requested versions.
- **Registry** — Browse the full mise plugin registry. Installed tools are marked with a checkmark. Shows backend and aliases columns.
- **Tasks** — List all mise tasks defined in your project, with descriptions and source file. Press `Enter` to run a task.
- **Environment** — Inspect all environment variables exported by mise, including their value, source file, and which tool set them.
- **Settings** — Browse and search all mise settings with their current values and types.
- **Config** — Inspect your mise config files and the tools they define. Press `t` to trust a config file.
- **Doctor** — Run `mise doctor` diagnostics directly in the TUI.

### Search

- Press `/` to enter search mode — fuzzy filtering with ranked results across all tabs.
- Navigate results with `j`/`k` while searching.
- Matched characters are highlighted in the results.
- Press `Enter` or `Esc` to exit search.

### Tool management

- **Install** (`i`) — Pick a version from the registry and install it.
- **Use globally** (`U`) — Set a tool version globally via `mise use --global`.
- **Uninstall** (`d`) — Remove an installed tool version.
- **Update** (`u`) — Update a tool to its latest version.
- **Upgrade all** (`U` on Outdated tab) — Upgrade every outdated tool at once.
- **Prune** (`p`) — Preview and remove unused tool versions (`mise prune`).

### Quality of life

- **Refresh** (`r`) — Reload all data from mise without restarting.
- **Column sorting** (`s`) — Cycle through sort columns and toggle ascending/descending order. Active column shown with ▲/▼ in the header.
- **Mouse support** — Scroll with the mouse wheel; click the sidebar to switch tabs.
- **Version picker search** — Filter the version list while picking a version to install or use.
- **Spinner / progress** — Long-running operations show a progress indicator.
- **Status messages** — Operation results appear briefly at the bottom of the screen.

## Prerequisites

- [mise](https://mise.jdx.dev) must be installed and available in your `PATH`

## Installation

### From source

```sh
git clone https://github.com/IhsenBouallegue/misetui.git
cd misetui
cargo install --path .
```

### With cargo

```sh
cargo install misetui
```

## Usage

```sh
misetui
```

## Keybindings

### Global

| Key | Action |
|-----|--------|
| `j` / `k` | Move down / up |
| `Page Down` / `Page Up` | Move down / up 10 rows |
| `h` / `l` | Focus sidebar / content |
| `Tab` / `Shift+Tab` | Next / previous tab |
| `/` | Enter search mode |
| `Esc` | Cancel search / close popup |
| `r` | Refresh all data |
| `s` | Cycle sort column / toggle direction |
| `?` | Show help |
| `q` / `Ctrl+c` | Quit |

### Tools tab

| Key | Action |
|-----|--------|
| `Enter` | Show tool detail |
| `u` | Update selected tool |
| `d` | Uninstall selected tool |

### Registry tab

| Key | Action |
|-----|--------|
| `i` | Install selected tool (version picker) |
| `U` | Use selected tool globally (version picker) |

### Outdated tab

| Key | Action |
|-----|--------|
| `u` | Upgrade selected tool |
| `U` | Upgrade all outdated tools |

### Tasks tab

| Key | Action |
|-----|--------|
| `Enter` | Run selected task |

### Config tab

| Key | Action |
|-----|--------|
| `t` | Trust selected config file |

### Search mode

| Key | Action |
|-----|--------|
| Type | Filter results (fuzzy, ranked) |
| `j` / `k` | Navigate filtered results |
| `Enter` / `Esc` | Exit search |

## License

[MIT](LICENSE)
