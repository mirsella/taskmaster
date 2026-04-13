# taskmaster

Rust process manager inspired by `supervisor`, with a terminal UI and TOML configuration files.

It can launch multiple programs, apply restart policies, and log output to files or journald.

## Run

```bash
cargo run -- config/default.toml
```

## Test

```bash
cargo test
```

## Minimal config example

```toml
[[program]]
name = "date"
command = "date"
```

![terminal ui screenshot](https://github.com/mirsella/taskmaster/assets/45905567/47b97736-9987-490f-89a0-3fd204137151)
