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
    fmt,
    fs::File,
    io,
    path::{Path, PathBuf},
    process::{self, Command, Stdio},
    time::{Duration, Instant},
};
use tracing::{debug, error, info, warn};

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
    pub stdout_append: bool,
    #[serde(default)]
    pub stderr_append: bool,
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
    fn create_child(&mut self, cmd: &mut process::Command) -> io::Result<process::Child> {
        let setup_io = |path: Option<&Path>, append: bool| {
            path.map_or(Ok(Stdio::null()), |path| {
                File::options()
                    .append(append)
                    .create(true)
                    .open(path)
                    .map(Stdio::from)
            })
        };
        let stdin = setup_io(self.stdin.as_deref(), false)?;
        let stdout = setup_io(self.stdout.as_deref(), self.stdout_append)?;
        let stderr = setup_io(self.stderr.as_deref(), self.stderr_append)?;

        let mut env_vars = HashMap::new();
        for entry in self.env.clone() {
            let parts: Vec<&str> = entry.splitn(2, '=').collect();
            if parts.len() == 2 {
                env_vars.insert(parts[0].to_string(), parts[1].to_string());
            } else {
                warn!("Invalid environment variable entry: {}", entry);
            }
        }

        let cwd = match &self.cwd {
            Some(path) => PathBuf::from(path),
            None => current_dir().unwrap_or_default(),
        };

        cmd.stdin(stdin)
            .stdout(stdout)
            .stderr(stderr)
            .args(self.args.clone())
            .envs(env_vars)
            .current_dir(cwd)
            .spawn()
    }

    pub fn launch(&mut self) {
        for process_nb in 1..self.processes + 1 {
            let mut new_process = Command::new(self.command.clone());
            let new_child = match self.start_policy {
                StartPolicy::Auto => match self.create_child(&mut new_process) {
                    Ok(new_child) => Child {
                        process: Process::Running(new_child),
                        start_time: Some(Instant::now()),
                        status: ChildStatus::Running,
                    },
                    Err(e) => {
                        error!("Error while creating child: {e}");
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
                                info!("{pre_string} The process has already exited [{status}]")
                            }
                            None => {
                                info!(
                                    "{pre_string} Sinding {} to the process {}",
                                    self.stop_signal,
                                    c.id()
                                );
                                unsafe { kill(c.id() as i32, self.stop_signal as i32) };
                                // I'll look into the timeout later
                            }
                        },
                        Err(e) => {
                            error!("{pre_string} Error while trying to get child information: {e}")
                        }
                    }
                }
                Process::NotRunning(_c) => info!("{pre_string} The process was not running"),
            }
        }
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
