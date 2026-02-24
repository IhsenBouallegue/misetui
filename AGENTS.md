# Agent Instructions

## mise — Dev Tool Version Manager

This project uses [mise](https://mise.jdx.dev/) to manage tool versions.

### Installing tools
```
mise install
```

### Running tasks
```
mise run <task-name>
mise tasks ls
```

### Checking environment
```
mise status
mise ls
```

### Pinned versions
Tool versions are pinned in `.mise.toml`. Always use the pinned version when running tools.
Do not upgrade tool versions without updating `.mise.toml`.
