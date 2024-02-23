/* ************************************************************************** */
/*                                                                            */
/*                                                        :::      ::::::::   */
/*   mod.rs                                             :+:      :+:    :+:   */
/*                                                    +:+ +:+         +:+     */
/*   By: nguiard <nguiard@student.42.fr>            +#+  +:+       +#+        */
/*                                                +#+#+#+#+#+   +#+           */
/*   Created: 2024/02/22 10:40:09 by nguiard           #+#    #+#             */
/*   Updated: 2024/02/22 19:26:04 by nguiard          ###   ########.fr       */
/*                                                                            */
/* ************************************************************************** */

use crate::config::Signal;
use libc::kill;
use serde::Deserialize;
use serde_with::{serde_as, DurationSeconds};
use std::{
    collections::HashMap,
    env::current_dir,
    error::Error,
    fmt,
    fs::{File, OpenOptions},
    path::{Path, PathBuf},
    process::{self, Command, Stdio},
    time::{Duration, Instant},
};
use tracing::{debug, error, info, instrument, trace, warn};

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

#[derive(Debug, Clone, PartialEq, Eq, Copy)]
pub enum ChildStatus {
    Stopped,
    Running,
    Waiting,
    Crashed,
}

impl fmt::Display for ChildStatus {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            ChildStatus::Stopped => write!(f, "\x1b[33mStopped\x1b[0m"),
            ChildStatus::Running => write!(f, "\x1b[32mRunning\x1b[0m"),
            ChildStatus::Waiting => write!(f, "Waiting"),
            ChildStatus::Crashed => write!(f, "\x1b[31mCrashed\x1b[0m"),
        }
    }
}

#[derive(Debug)]
pub enum Process {
    NotRunning(process::Command),
    Running(process::Child),
}

#[derive(Debug)]
pub struct Child {
    pub process: Process,
    pub start_time: Option<Instant>,
    pub status: ChildStatus,
}

#[serde_as]
#[derive(Deserialize, Debug)]
pub struct Program {
    // Mandatory
    pub command: PathBuf,

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
    pub valid_exit_codes: Vec<u8>,
    #[serde(default)]
    pub restart_policy: RestartPolicy,
    pub max_restarts: Option<u32>,
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
}
fn default_processes() -> u8 {
    1
}
fn default_timeout() -> Duration {
    Duration::from_secs(10)
}
pub fn generate_name() -> String {
    names::Generator::default().next().unwrap()
}

impl Program {
    #[instrument(skip_all)]
    fn create_child(
        &mut self,
        cmd: &mut process::Command,
    ) -> Result<process::Child, Box<dyn Error>> {
        let setup_io = |path: Option<&Path>, file_options: &mut OpenOptions| {
            path.map_or(Ok(Stdio::null()), |path| {
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
        trace!(name = self.name, config = ?self.stdout, "Setting up stdout");
        let stdout = setup_io(
            self.stdout.as_deref(),
            File::options()
                .write(true)
                .truncate(self.stdout_truncate)
                .create(true),
        )?;
        trace!(name = self.name, config = ?self.stderr, "Setting up stderr");
        let stderr = setup_io(
            self.stderr.as_deref(),
            File::options()
                .write(true)
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

        let cwd = match &self.cwd {
            Some(path) => PathBuf::from(path),
            None => current_dir().unwrap_or_default(),
        };
        Ok(cmd
            .stdin(stdin)
            .stdout(stdout)
            .stderr(stderr)
            .args(self.args.clone())
            .envs(env_vars)
            .current_dir(cwd)
            .spawn()?)
    }

    pub fn launch(&mut self) {
        for process_nb in 1..=self.processes {
            let mut new_process = Command::new(self.command.clone());
            let new_child = match self.start_policy {
                StartPolicy::Auto => match self.create_child(&mut new_process) {
                    Ok(new_child) => Child {
                        process: Process::Running(new_child),
                        start_time: Some(Instant::now()),
                        status: ChildStatus::Running,
                    },
                    Err(e) => {
                        error!("creating child: {e}");
                        Child {
                            process: Process::NotRunning(new_process),
                            start_time: None,
                            status: ChildStatus::Crashed,
                        }
                    }
                },
                StartPolicy::Manual => Child {
                    process: Process::NotRunning(new_process),
                    start_time: None,
                    status: ChildStatus::Waiting,
                },
            };
            match (&new_child.process, &new_child.status) {
                (_, ChildStatus::Crashed) => {}
                (Process::Running(chd), _) => info!(
                    "{} ({}): Child {} now running. [{}]",
                    self.name,
                    self.command.display(),
                    process_nb,
                    chd.id()
                ),
                (Process::NotRunning(_cmd), _) => debug!(
                    "{} ({}): Child {} loaded.",
                    self.name,
                    self.command.display(),
                    process_nb
                ),
            }
            self.childs.push(new_child);
        }
    }

    pub fn kill(&mut self) {
        let pre_string = format!("{} ({}):", self.name, self.command.display());
        for child in &mut self.childs {
            match &mut child.process {
                Process::Running(ref mut c) => {
                    match c.try_wait() {
                        Ok(res) => match res {
                            Some(status) => {
                                debug!("{pre_string} The process has already exited [{status}]")
                            }
                            None => {
                                debug!(
                                    "{pre_string} Sending {} to the process {}",
                                    self.stop_signal,
                                    c.id()
                                );
                                unsafe { kill(c.id() as i32, self.stop_signal as i32) };
                                // I'll look into the timeout later
                                // lucas: faudrait un flag pour chaque Child pour savoir si on a déjà envoyé un signal et a quel Instant
                                // pour que la prochaine fois on fasse un check si le child est en cours de shutdown, et si il faut le fermer de force car il a timeout
                            }
                        },
                        Err(e) => {
                            error!("{pre_string} trying to get child information: {e}")
                        }
                    }
                }
                Process::NotRunning(_c) => debug!("{pre_string} The process was not running"),
            }
        }
        info!("{pre_string} All children have been stopped");
    }

    pub fn status(&self, all: bool) {
        match all {
            true => println!(
                "Program: {}\ncmd: {}\nargs: {:?}",
                self.name,
                self.command.display(),
                self.args
            ),
            false => println!("Program: {}", self.name),
        }
        println!("PID     | Status  | Uptime");
        for child in &self.childs {
            match &child.process {
                Process::Running(p) => print!("{:<width$}|", p.id(), width = 8),
                Process::NotRunning(_) => print!("None    |"),
            }
            print!(" {:<width$} |", child.status, width = 8);
            match child.start_time {
                Some(time) => println!(" {:?}", Instant::now() - time),
                None => println!(" Unknown"),
            }
        }
    }
}
