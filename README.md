# misetui

A terminal user interface for [mise](https://mise.jdx.dev) — manage your dev tools, versions, and configs without leaving the terminal.

Built with [Ratatui](https://ratatui.rs) and async Rust.

## Features

- **Tools** — View all installed tools with version, status, and source info
- **Registry** — Browse the full mise registry, search for tools, and install specific versions
- **Config** — Inspect your mise config files and their tool definitions
- **Doctor** — Run `mise doctor` diagnostics directly in the TUI
- **Search** — Fuzzy-style filtering across all tabs — just start typing
- **Install / Uninstall / Update** — Manage tool versions with keyboard shortcuts

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

| Key | Action |
|---|---|
| `j` / `k` | Move down / up |
| `h` / `l` | Focus sidebar / content |
| `Tab` / `Shift+Tab` | Next / previous tab |
| `/` | Enter search mode |
| `Esc` | Cancel search / close popup |
| `i` | Install tool (Registry tab) |
| `d` | Uninstall tool (Tools tab) |
| `u` | Update tool (Tools tab) |
| `Enter` | Confirm action |
| `?` | Show help |
| `q` / `Ctrl+c` | Quit |

In search mode, just type to filter — press `Enter` or `Esc` to exit search.

## License

[MIT](LICENSE)
