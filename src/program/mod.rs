/* ************************************************************************** */
/*                                                                            */
/*                                                        :::      ::::::::   */
/*   mod.rs                                             :+:      :+:    :+:   */
/*                                                    +:+ +:+         +:+     */
/*   By: nguiard <nguiard@student.42.fr>            +#+  +:+       +#+        */
/*                                                +#+#+#+#+#+   +#+           */
/*   Created: 2024/02/22 10:40:09 by nguiard           #+#    #+#             */
/*   Updated: 2024/02/23 18:43:19 by nguiard          ###   ########.fr       */
/*                                                                            */
/* ************************************************************************** */

pub mod child;

use crate::config::Signal;
use child::{Child, ChildStatus::*};
use libc::kill;
use serde::Deserialize;
use serde_with::{serde_as, DurationSeconds};
use std::{
    collections::HashMap,
    env::current_dir,
    error::Error,
    fmt::Write,
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
    #[serde(default = "default_max_restarts")]
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
    #[serde(default = "default_process_command")]
    #[serde(skip)]
    pub process_command: Option<process::Command>,
}
fn default_processes() -> u8 {
    1
}
fn default_timeout() -> Duration {
    Duration::from_secs(10)
}
pub fn default_max_restarts() -> isize {
    10
}
pub fn default_process_command() -> Option<process::Command> {
    None
}
pub fn generate_name() -> String {
    names::Generator::default().next().unwrap()
}

impl Program {
    fn set_command(&mut self) {
        match &self.process_command {
            Some(_) => {}
            None => self.process_command = Some(process::Command::new(&self.command)),
        }
    }

    #[instrument(skip_all)]
    fn create_child(&mut self, cmd: &mut process::Command) -> Result<Child, Box<dyn Error>> {
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
                .ok_or(format!("Invalid env var: {entry}"))
                .unwrap(); // CHANGE
            env_vars.insert(parts.0.to_string(), parts.1.to_string());
        }

        let cwd = self.cwd.clone().unwrap_or(
            current_dir().map_err(|e| format!("couldn't get the current directory: {e}"))?,
        );

        match cmd
            .stdin(stdin)
            .stdout(stdout)
            .stderr(stderr)
            .args(self.args.clone())
            .envs(env_vars)
            .current_dir(cwd)
            .spawn()
        {
            Ok(new_child) => Ok(Child::new(new_child)),
            Err(e) => {
                error!("Error while creating child: {e}");
                Err(e.into())
            }
        }
    }

    pub fn start(&mut self) -> Result<(), Box<dyn Error>> {
        self.set_command();
        for process_nb in 1..=self.processes {
            let mut new_process = Command::new(self.command.clone());
            let new_child = match self.start_policy {
                StartPolicy::Auto => self.create_child(&mut new_process)?,
                StartPolicy::Manual => Child {
                    process: None,
                    last_update: Instant::now(),
                    status: Waiting,
                    restarts: 0,
                },
            };
            match (&new_child.process, &new_child.status) {
                (_, Crashed) => {}
                (Some(chd), _) => info!(
                    "{} ({}): Child {} now running. [{}]",
                    self.name,
                    self.command.display(),
                    process_nb,
                    chd.id()
                ),
                (None, _) => debug!(
                    "{} ({}): Child {} loaded.",
                    self.name,
                    self.command.display(),
                    process_nb
                ),
            }
            self.childs.push(new_child);
        }
        Ok(())
    }

    /// Kill the program and all its children. for graceful shutdown, check stop().
    // TODO: `kill` as the name suggest, should force kill the childs
    pub fn kill(&mut self) {
        let pre_string = format!("{} ({}):", self.name, self.command.display());
        for child in &mut self.childs {
            match &mut child.process {
                Some(ref mut c) => match c.try_wait() {
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
                        }
                    },
                    Err(e) => {
                        error!("{pre_string} trying to get child information: {e}")
                    }
                },
                None => debug!("{pre_string} The process was not running"),
            }
        }
        info!("{pre_string} All children have been stopped");
    }

    /// Returns updates a supposedly running child
    fn update_running_child(&self, child: &mut Child) {
        let process_child = child.process.as_mut().unwrap();
        let pid = process_child.id();
        match process_child.try_wait() {
            Ok(res) => match res {
                Some(exit_code) => match exit_code.code() {
                    Some(code) => {
                        if self.valid_exit_codes.contains(&(code as u8)) {
                            info!("Child {pid} exited successfully");
                            child.status = Stopped;
                            child.last_update = Instant::now();
                        } else {
                            info!("Child {pid} exited with a status code of {exit_code} which is unexpected");
                            child.restarts += 1;
                            if child.restarts == self.max_restarts {
                                child.status = MaxRestarts;
                                child.last_update = Instant::now();
                            } else {
                                child.status = Crashed;
                                child.last_update = Instant::now();
                            }
                        }
                    }
                    None => {
                        info!("Child {pid} has been killed by a signal");
                        child.status = Crashed;
                        child.last_update = Instant::now();
                    }
                },
                None => {}
            },
            Err(e) => {
                error!("Could not wait for process {}: {e}", process_child.id());
                child.status = Unknown;
                child.last_update = Instant::now();
            }
        };
    }

    fn try_force_kill(&self, child: &mut Child) {
        if (Instant::now() - child.last_update) > self.graceful_timeout {
            match &mut child.process {
                Some(c) => {
                    child.last_update = Instant::now();
                    child.status = SentSIGKILL;
                    match &c.kill() {
                        Ok(_) => {}
                        Err(e) => error!("Could not kill process {}: {e}", c.id()),
                    }
                }
                None => {
                    warn!("Tried to kill a process that is not running");
                    child.last_update = Instant::now();
                    child.status = Stopped;
                }
            }
        }
    }

    fn update_sigkill(&self, child: &mut Child) {
        match &mut child.process {
            Some(c) => {
                match c.try_wait() {
                    Ok(Some(exit)) => {
                        //restart
                        child.last_update = Instant::now();
                        child.status = Stopped;
                    }
                    Ok(None) => {
                        child.last_update = Instant::now();
                        child.status = Stopped;
                    }
                    Err(e) => {}
                }
            }
            None => {
                child.last_update = Instant::now();
                child.status = Stopped;
            }
        }
    }

    /// TODO: start the graceful shutdown of the childs: send the stop signal, and mark them as stopping
    pub fn stop(&mut self) {
        todo!()
    }
    /// TODO: mark the program to be restarted
    pub fn restart(&mut self) {
        todo!()
    }
    /// TODO: apply a new config to the program, and restart if needed
    pub fn update(&mut self, new: Program) {
        todo!()
    }
    /// TODO: this is the main function that will be called by the main loop.
    /// it will check the status of the children, and:
    /// - force kill if the graceful kill timeout is reached
    /// - start childs if needed while in restart, or crash and if it should
    pub fn tick(&mut self) {
        for i in 0..self.childs.len() {
            let mut clone = self.childs[i].my_take();
            match (&clone.process, self.childs[i].status) {
                (Some(_), Running) => self.update_running_child(&mut clone),
                (Some(_), BeingKilled) => self.try_force_kill(&mut clone),
                (Some(c), SentSIGKILL) => {} // Try to update the data
                (Some(c), Unknown) => {}     // Try to get data
                (Some(c), Crashed) => {}     // restart if possible
                (Some(c), _) => {}           // Should not do anything
                (None, _) => {}
            }
            self.childs[i] = clone;
        }
    }

    // FIX: this function is only for dev/debug, i will do something cleaner later
    pub fn status(&self) -> String {
        let mut buffer = String::new();
        writeln!(buffer, "Program: {}", self.name);
        writeln!(buffer, "PID     | Status  | Since");
        for child in &self.childs {
            match &child.process {
                Some(p) => write!(buffer, "{:<width$}|", p.id(), width = 8),
                None => write!(buffer, "None    |"),
            };
            write!(buffer, " {:<width$?} |", child.status, width = 8);
            writeln!(buffer, " {:?}", Instant::now() - child.last_update);
        }
        writeln!(buffer);
        buffer
    }
}

#[cfg(test)]
mod parsing_tests {

    #[test]
    fn default() {}
}
