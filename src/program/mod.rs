/* ************************************************************************** */
/*                                                                            */
/*                                                        :::      ::::::::   */
/*   mod.rs                                             :+:      :+:    :+:   */
/*                                                    +:+ +:+         +:+     */
/*   By: nguiard <nguiard@student.42.fr>            +#+  +:+       +#+        */
/*                                                +#+#+#+#+#+   +#+           */
/*   Created: 2024/02/22 10:40:09 by nguiard           #+#    #+#             */
/*   Updated: 2024/02/22 16:41:30 by nguiard          ###   ########.fr       */
/*                                                                            */
/* ************************************************************************** */

use std::{collections::HashMap, env::current_dir, fs::{File, OpenOptions}, path::Path, process::{self, exit, Command, Stdio}};
use libc::kill;
use log::{debug, error, info};

use crate::config::signal::Signal;
use serde::Deserialize;
use serde_with::{serde_as, DurationSeconds};
use std::{
    path::PathBuf,
    time::{Duration, Instant},
};

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
	// Mendatory
    pub command: String,
	pub name: String,

	// Optional
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
    pub valid_signal: Signal,
    #[serde(default)]
    #[serde_as(as = "DurationSeconds<u64>")]
    pub graceful_timeout: Duration,
    pub stdin: Option<String>,
    pub stderr: Option<String>,
    pub stdout: Option<String>,
    #[serde(default = "default_false")]
    pub stdout_append: bool,
	#[serde(default = "default_false")]
    pub stderr_append: bool,
    #[serde(default)]
    pub args: Vec<String>,
    #[serde(default)]
    pub env: Vec<String>,
    pub cwd: Option<String>,
    pub umask: Option<String>,
	pub user: Option<String>,

    // runtime only
    #[serde(skip)]
    pub childs: Vec<Child>,
}

fn default_processes() -> u8 {
    1
}

fn default_false() -> bool {
    false
}

impl Program {
	fn open_file(path: String, read: bool, write: bool, append: bool, create: bool)
		-> Stdio {
		match OpenOptions::new()
						.read(read)
						.write(write)
						.append(append)
						.create(create)
						.open(&path) {
			Ok(file) => Stdio::from(file),
			Err(e) => {
				error!("Could not open {}: {e}", path);
				Stdio::null()
			}
		}
	}
	
	fn create_child(&mut self, mut cmd: process::Command) -> process::Child {
		let stdin_path = self.stdout.clone().unwrap_or("/dev/stdin".into());
		let stdout_path = self.stdout.clone().unwrap_or("/dev/stdout".into());
		let stderr_path = self.stderr.clone().unwrap_or("/dev/stderr".into());
		
		let stdin: Stdio = Program::open_file(stdin_path, true,
												false, false, false);
		let stdout: Stdio = Program::open_file(stdout_path, false,
												true, self.stdout_append, true);
		let stderr: Stdio = Program::open_file(stderr_path, false,
												true, self.stderr_append, true);

		let mut env_vars = HashMap::new();
		for entry in self.env.clone() {
			let parts: Vec<&str> = entry.splitn(2, '=').collect();
			if parts.len() == 2 {
				env_vars.insert(parts[0].to_string(), parts[1].to_string());
			} else {
				error!("Invalid environment variable entry: {}", entry);
			}
		}

		let cwd = match &self.cwd {
			Some(path) => PathBuf::from(path),
			None => current_dir().unwrap_or(PathBuf::new()),
		};

		cmd.stdin(stdin).stdout(stdout).stderr(stderr)
			.args(self.args.clone()).envs(env_vars)
			.current_dir(cwd)
			.spawn()
			.expect("Problem in command execution")
	}
	
	pub fn launch(&mut self) {
		for _process_nb in 1..self.processes + 1 {
			let new_process = Command::new(self.command.clone());
			let new_child = match self.start_policy {
				StartPolicy::Auto => Child {
					process: Process::Running(self.create_child(new_process)),
					start_time: Some(Instant::now()),
					status: ChildStatus::Running,
				},
				StartPolicy::Manual => Child {
					process: Process::NotRunning(new_process),
					start_time: None,
					status: ChildStatus::Waiting,
				}	
			};
			self.childs.push(new_child);
		}
	}

	pub fn kill(&mut self) {
		for child in &mut self.childs {
			match &mut child.process {
				Process::Running(ref mut c) => {
					match c.try_wait() {
						Ok(res) => match res {
							Some(status) => info!("The process has already exited [{status}]"),
							None => {
								info!("Sinding {} to the process", self.valid_signal);
								unsafe {kill(c.id() as i32, self.valid_signal as i32)};
								// I'll look into the timeout later
							}
						},
						Err(e) => error!("Error while trying to get child information: {e}"),
					}
				}
				Process::NotRunning(_c) => info!("The process was not running"),
			}
		}
	}
}