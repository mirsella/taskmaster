/* ************************************************************************** */
/*                                                                            */
/*                                                        :::      ::::::::   */
/*   mod.rs                                             :+:      :+:    :+:   */
/*                                                    +:+ +:+         +:+     */
/*   By: nguiard <nguiard@student.42.fr>            +#+  +:+       +#+        */
/*                                                +#+#+#+#+#+   +#+           */
/*   Created: 2024/02/22 10:40:09 by nguiard           #+#    #+#             */
/*   Updated: 2024/02/22 15:00:23 by nguiard          ###   ########.fr       */
/*                                                                            */
/* ************************************************************************** */

use std::{collections::HashMap, env::current_dir, fs::{File, OpenOptions}, path::Path, process::{self, exit, Command, Stdio}};
use libc::kill;

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
    pub stdin: Option<PathBuf>,
    pub stdout: Option<PathBuf>,
    #[serde(default)]
    pub args: Vec<String>,
    #[serde(default)]
    pub env: Vec<String>,
    pub cwd: Option<PathBuf>,
    pub umask: Option<String>,
	pub user: Option<String>,

    // runtime only
    #[serde(skip)]
    pub childs: Vec<Child>,
}

fn default_processes() -> u8 {
    1
}

impl Program {
	fn create_child(&mut self, mut cmd: process::Command) -> process::Child {
		// let stdin: Stdio = match OpenOptions::new()
		// 							.write(true)
		// 							.open(self.stdin.take().unwrap_or(PathBuf::from("/dev/null"))) {
		// 	Ok(file) => Stdio::from(file),
		// 	Err(e) => {
		// 		eprintln!("Could not open stdin: {e}");
		// 		Stdio::piped()
		// 	}
		// };

		// let stdout: Stdio = match OpenOptions::new()
		// 							.read(true)
		// 							.open(self.stdout.take().unwrap_or(PathBuf::from("/dev/null"))) {
		// 		Ok(file) => Stdio::from(file),
		// 		Err(e) => {
		// 		eprintln!("Could not open stdout: {e}");
		// 		Stdio::piped()
		// 	}
		// };

		let stdin = Stdio::inherit();
		let stdout = Stdio::inherit();

		let mut env_vars = HashMap::new();
		for entry in self.env.clone() {
			let parts: Vec<&str> = entry.splitn(2, '=').collect();
			if parts.len() == 2 {
				env_vars.insert(parts[0].to_string(), parts[1].to_string());
			} else {
				eprintln!("Invalid environment variable entry: {}", entry);
			}
		}

		dbg!(&stdin);
		dbg!(&stdout);

		cmd.stdin(stdin).stdout(stdout)
			.args(self.args.clone()).envs(env_vars)
			.current_dir(self.cwd.take()
							.unwrap_or(current_dir()
							.unwrap_or(PathBuf::from("/"))))
			.spawn()
			.expect("Problem in command execution")
	}
	
	pub fn launch(&mut self) {
		for _process_nb in 1..self.processes + 1 {
			let new_process = Command::new(self.command.clone());
			let new_child = Child {
				process: match self.start_policy {
					StartPolicy::Auto => Process::Running(self.create_child(new_process)),
					StartPolicy::Manual => Process::NotRunning(new_process),
				},
				start_time: match self.start_policy {
					StartPolicy::Auto => Some(Instant::now()),
					StartPolicy::Manual => None,
				},
				status: match self.start_policy {
					StartPolicy::Auto => ChildStatus::Running,
					StartPolicy::Manual => ChildStatus::Waiting,
				},
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
							Some(status) => println!("The process has already exited [{status}]"),
							None => {
								println!("Sinding {} to the process", self.valid_signal);
								unsafe {kill(c.id() as i32, self.valid_signal as i32)};
								// I'll look into the timeout later
							}
						},
						Err(e) => eprintln!("Error while trying to get child information: {e}"),
					}
				}
				Process::NotRunning(_c) => println!("The process was not running"),
			}
		}
	}
}