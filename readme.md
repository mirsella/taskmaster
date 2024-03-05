# program management like supervisor
example configuration file:
```toml
loglevel = "trace"

[[program]]
name = "short"
command = "date"
processes = 1
start_policy = "auto"
valid_exit_codes = [0]
stdout = "./test.log"
stdout_truncate = false

[[program]]
name = "long"
command = "sleep"
args = ["15"]

[[program]]
name = "never ending"
command = "yes"
```

![terminal ui screenshot](https://github.com/mirsella/taskmaster/assets/45905567/47b97736-9987-490f-89a0-3fd204137151)

# school bonus

- Launch a program as another user
- Advanced logging (stdout, file, journald)
- Configurable log level from config with runtime reload, or with RUST_LOG
- Tests and CI
- Terminal interface
