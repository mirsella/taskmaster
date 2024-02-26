/* ************************************************************************** */
/*                                                                            */
/*                                                        :::      ::::::::   */
/*   mod.rs                                             :+:      :+:    :+:   */
/*                                                    +:+ +:+         +:+     */
/*   By: nguiard <nguiard@student.42.fr>            +#+  +:+       +#+        */
/*                                                +#+#+#+#+#+   +#+           */
/*   Created: 2024/02/22 10:40:09 by nguiard           #+#    #+#             */
/*   Updated: 2024/02/26 13:35:36 by nguiard          ###   ########.fr       */
/*                                                                            */
/* ************************************************************************** */

pub mod child;

use crate::config::Signal;
use child::{Child, Status};
use serde::Deserialize;
use serde_with::{serde_as, DurationSeconds};
use std::{
    collections::HashMap,
    env::current_dir,
    error::Error,
    fs::{self, File, OpenOptions},
    mem,
    path::{Path, PathBuf},
    process::{self, Command, Stdio},
    time::Duration,
};
use tracing::{debug, info, instrument, trace};

#[derive(Deserialize, Debug, Default, PartialEq, Clone, Eq)]
#[serde(rename_all = "lowercase")]
pub enum RestartPolicy {
    #[default]
    Never,
    Always,
    UnexpectedExit,
}

#[derive(Deserialize, Debug, Default, PartialEq, Clone, Eq)]
#[serde(rename_all = "lowercase")]
pub enum StartPolicy {
    #[default]
    Auto,
    Manual,
}

#[serde_as]
#[derive(Deserialize, Debug)]
pub struct Program {
    // Mandatory
    #[serde(rename = "command")]
    pub cmd: PathBuf,

    // Optional
    #[serde(default = "generate_name")]
    pub name: String,
    #[serde(default)]
    pub start_policy: StartPolicy,
    #[serde(default = "default_processes")]
    pub processes: u8,
    #[serde(default)]
    #[serde_as(as = "DurationSeconds<u64>")]
    pub min_runtime: Duration,
    #[serde(default)]
    pub valid_exit_codes: Vec<i32>,
    #[serde(default)]
    pub restart_policy: RestartPolicy,
    #[serde(default = "default_max_restarts")]
    /// -1 means infinite restarts
    pub max_restarts: isize,
    #[serde(default)]
    pub stop_signal: Signal,
    #[serde(default = "default_timeout")]
    #[serde_as(as = "DurationSeconds<u64>")]
    pub graceful_timeout: Duration,
    pub stdin: Option<PathBuf>,
    pub stderr: Option<PathBuf>,
    pub stdout: Option<PathBuf>,
    #[serde(default)]
    pub stdout_truncate: bool,
    #[serde(default)]
    pub stderr_truncate: bool,
    #[serde(default)]
    pub args: Vec<String>,
    #[serde(default)]
    pub env: Vec<String>,
    pub cwd: Option<PathBuf>,
    pub umask: Option<u32>,
    pub user: Option<String>,

    // runtime only
    #[serde(skip)]
    pub childs: Vec<Child>,
    #[serde(skip)]
    pub force_restart: bool,
    #[serde(skip)]
    pub update_asked: bool,
}
fn default_processes() -> u8 {
    1
}
fn default_timeout() -> Duration {
    Duration::from_secs(10)
}
pub fn default_max_restarts() -> isize {
    3
}
pub fn generate_name() -> String {
    names::Generator::default().next().unwrap()
}

fn is_our_fd(path: &Path) -> Result<bool, Box<dyn Error>> {
    let resolved_path = fs::canonicalize(path)?;
    let resolved_path = resolved_path
        .to_str()
        .ok_or("the path is invalid unicode")?;
    let proc_paths = ["/proc/self/fd", &format!("/proc/{}/fd", process::id())];

    if proc_paths.iter().any(|p| resolved_path.contains(p)) {
        return Ok(true);
    }
    Ok(false)
}

impl Program {
    #[instrument(skip_all)]
    fn create_child(&mut self) -> Result<Child, Box<dyn Error>> {
        let setup_io = |path: Option<&Path>, file_options: &mut OpenOptions| {
            path.map_or(Ok(Stdio::null()), |path| {
                if is_our_fd(path)
                    .map_err(|e| format!("checking if the path is our own stdio fd: {e}"))?
                {
                    return Err("File points to our own stdio file descriptor".into());
                };
                file_options
                    .open(path)
                    .map_err(|e| format!("opening file `{path:?}`: {e}"))
                    .map(Stdio::from)
            })
        };
        trace!(name = self.name, config = ?self.stdin, "Setting up stdin");
        let stdin = setup_io(
            self.stdin.as_deref(),
            File::options().read(true).create(false),
        )?;
        trace!(
            name = self.name,
            config = ?self.stdout.as_ref().unwrap_or(&"null".into()),
            "Setting up stdout"
        );
        let stdout = setup_io(
            self.stdout.as_deref(),
            File::options()
                .append(true)
                .truncate(self.stdout_truncate)
                .create(true),
        )?;
        trace!(name = self.name, config = ?self.stderr, "Setting up stderr");
        let stderr = setup_io(
            self.stderr.as_deref(),
            File::options()
                .append(true)
                .truncate(self.stderr_truncate)
                .create(true),
        )?;
        trace!(name = self.name, "Setting up stdio done");

        let mut env_vars = HashMap::new();
        for entry in self.env.clone() {
            let parts = entry
                .split_once('=')
                .ok_or(format!("Invalid env var: {entry}"))?;
            env_vars.insert(parts.0.to_string(), parts.1.to_string());
        }

        let cwd = self.cwd.clone().unwrap_or(
            current_dir().map_err(|e| format!("couldn't get the current directory: {e}"))?,
        );

        let child = Command::new(&self.cmd)
            .stdin(stdin)
            .stdout(stdout)
            .stderr(stderr)
            .args(self.args.clone())
            .envs(env_vars)
            .current_dir(cwd)
            .spawn()?;
        debug!(pid = child.id(), name = self.name, "Running");
        Ok(Child::new(child))
    }

    #[instrument(skip_all)]
    pub fn start(&mut self) -> Result<(), Box<dyn Error>> {
        if self
            .childs
            .iter()
            .all(|c| matches!(c.status, Status::Finished(_, _) | Status::Stopped(_)))
        {
            self.childs.clear();
        } else {
            return Err("Some processes are still running".into());
        }
        info!(name = self.name, "starting process...");
        for _ in 0..self.processes {
            let child = self.create_child()?;
            self.childs.push(child);
        }
        info!(
            name = self.name,
            "all processes started ({})", self.processes
        );
        Ok(())
    }

    /// Kill the program and all its children. for graceful shutdown, check stop().
    #[instrument(skip_all)]
    pub fn kill(&mut self) {
        for child in &mut self.childs {
            debug!(
                pid = child.process.id(),
                name = self.name,
                signal = %self.stop_signal,
                "Killing"
            );
            let _ = child.process.kill();
        }
    }

    /// start the graceful shutdown of the childs: send the stop signal, and mark them as stopping
    pub fn stop(&mut self) {
        for child in &mut self.childs {
            if let Status::Running(_) | Status::Starting(_) = child.status {
                debug!(
                    pid = child.process.id(),
                    name = self.name,
                    signal = %self.stop_signal,
                    "Killing"
                );
                child.stop(self.stop_signal as i32);
            }
        }
    }
    /// mark the program to be restarted
    pub fn restart(&mut self) {
        self.force_restart = true;
        self.stop();
    }
    /// Applies a new config to the program, and restart if needed
    pub fn update(&mut self, new: Program) {
        if self.corresponds_to(&new) {
            info!("Not updating {}: configuration didn't change", self.name);
            return;
        }
        if self.update_asked {
            if self
                .childs
                .iter()
                .all(|c| matches!(c.status, Status::Finished(_, _) | Status::Stopped(_)))
            {
                self.childs.clear();
            }
            self.assign_new(new);
            self.update_asked = false;
            self.force_restart = false;
        } else {
            self.restart();
            self.update_asked = true;
        }
    }
    /// this need to be called regularly, to check the status of the program and its children.
    pub fn tick(&mut self) -> Result<(), Box<dyn Error>> {
        if self.force_restart
            && self
                .childs
                .iter()
                .all(|c| matches!(c.status, Status::Finished(_, _) | Status::Stopped(_)))
        {
            self.force_restart = false;
            self.childs.clear();
            self.start()?;
        }

        let finished_before = self
            .childs
            .iter()
            .all(|c| matches!(c.status, Status::Finished(_, _)));
        let mut childs = mem::take(&mut self.childs);
        for child in &mut childs {
            let _ = child.tick(self);
        }
        self.childs = childs;
        let finished_after = self
            .childs
            .iter()
            .all(|c| matches!(c.status, Status::Finished(_, _)));
        if !finished_before && finished_after {
            info!(
                name = self.name,
                "All ({}) processes finished", self.processes
            );
        }
        Ok(())
    }

    /// To check if two Programs have the same configuration
    pub fn corresponds_to(&self, other: &Program) -> bool {
        if self.name != other.name
            || self.cmd != other.cmd
            || self.processes != other.processes
            || self.min_runtime != other.min_runtime
            || self.valid_exit_codes != other.valid_exit_codes
            || self.max_restarts != other.max_restarts
            || self.stop_signal != other.stop_signal
            || self.graceful_timeout != other.graceful_timeout
            || self.stdin != other.stdin
            || self.stdout != other.stdout
            || self.stderr != other.stderr
            || self.stdout_truncate != other.stdout_truncate
            || self.stderr_truncate != other.stderr_truncate
            || self.args != other.args
            || self.env != other.env
            || self.cwd != other.cwd
            || self.umask != other.umask
            || self.user != other.user
            || self.start_policy != other.start_policy
        {
            return false;
        }
        true
    }

    pub fn assign_new(&mut self, other: Program) {
        self.name = other.name;
        self.cmd = other.cmd;
        self.processes = other.processes;
        self.min_runtime = other.min_runtime;
        self.valid_exit_codes = other.valid_exit_codes;
        self.max_restarts = other.max_restarts;
        self.stop_signal = other.stop_signal;
        self.graceful_timeout = other.graceful_timeout;
        self.stdin = other.stdin;
        self.stdout = other.stdout;
        self.stderr = other.stderr;
        self.stdout_truncate = other.stdout_truncate;
        self.stderr_truncate = other.stderr_truncate;
        self.args = other.args;
        self.env = other.env;
        self.cwd = other.cwd;
        self.umask = other.umask;
        self.user = other.user;
        self.start_policy = other.start_policy;
    }
}

#[cfg(test)]
mod program_tests {
    use std::{path::Path, process::id};

    use super::follow_link;
    use crate::config::Config;

    #[test]
    #[should_panic]
    fn open_stdin() {
        follow_link(Path::new("/dev/stdin"), id()).unwrap();
    }

    #[test]
    #[should_panic]
    fn open_stdout() {
        follow_link(Path::new("/dev/stdout"), id()).unwrap();
    }

    #[test]
    #[should_panic]
    fn open_stderr() {
        follow_link(Path::new("/dev/stderr"), id()).unwrap();
    }

    #[test]
    #[should_panic]
    fn open_self_zero() {
        follow_link(Path::new("/proc/self/fd/0"), id()).unwrap();
    }

    #[test]
    #[should_panic]
    fn open_self_one() {
        follow_link(Path::new("/proc/self/fd/1"), id()).unwrap();
    }

    #[test]
    #[should_panic]
    fn open_self_pid_zero() {
        follow_link(Path::new(format!("/proc/{}/fd/0", id()).as_str()), id()).unwrap();
    }

    #[test]
    #[should_panic]
    fn open_self_pid_one() {
        follow_link(Path::new(format!("/proc/{}/fd/1", id()).as_str()), id()).unwrap();
    }

    #[test]
    fn open_bash() {
        follow_link(Path::new("/bin/bash"), id()).unwrap();
    }

    #[test]
    fn open_basic_config() {
        follow_link(Path::new("config/default.toml"), id()).unwrap();
    }

    #[test]
    fn equal_configs() {
        let base = Config::load("config/default.toml").unwrap();
        let link = Config::load("config/default_link.toml").unwrap();

        for i in 0..base.program.len() {
            assert!(base.program[i].corresponds_to(&link.program[i]))
        }
    }

    #[test]
    #[should_panic]
    fn different_configs() {
        let base = Config::load("config/default.toml").unwrap();
        let diff = Config::load("config/default_diff.toml").unwrap();

        for i in 0..base.program.len() {
            assert!(base.program[i].corresponds_to(&diff.program[i]))
        }
    }
}

