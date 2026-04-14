# taskmaster

Rust process manager inspired by `supervisor`, with a terminal UI and TOML configuration files.

It can launch multiple programs, apply restart policies, and log output to files or journald.

## How It Works

At startup, `taskmaster` loads a TOML configuration file, spawns programs with `start_policy = "auto"`, and then keeps a main loop running for process supervision, signal handling, and TUI rendering.

Programs can define process count, restart behavior, exit policies, stdout/stderr targets, environment variables, working directory, user, umask, and stop signal handling.

The config can be reloaded at runtime, so the manager can update, stop, or start programs without restarting the whole application.

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

## Notes

- The bundled `config/default.toml` is more of a feature demo than a production-ready example.
- The TUI is designed for an interactive terminal.
- The project is Unix-oriented and uses signals and process control APIs directly.

![terminal ui screenshot](https://github.com/mirsella/taskmaster/assets/45905567/47b97736-9987-490f-89a0-3fd204137151)
