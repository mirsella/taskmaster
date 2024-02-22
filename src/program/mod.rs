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

use crate::config::signal::Signal;
use std::{collections::HashMap,
	env::current_dir, fmt,
	fs::OpenOptions,
	io,
	process::{self, Command, Stdio},
	path::PathBuf,
	time::{Duration, Instant},
};
use libc::kill;
use tracing::{debug, error, info, warn};
use serde::Deserialize;
use serde_with::{serde_as, DurationSeconds};

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
    pub stop_signal: Signal,
    #[serde(default = "default_timeout")]
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

fn default_timeout() -> Duration {
    Duration::from_secs(10)
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
	
	fn create_child(&mut self, cmd: &mut process::Command) -> io::Result<process::Child> {
		let stdin_path = self.stdin.clone().unwrap_or("/dev/stdin".into());
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
				warn!("Invalid environment variable entry: {}", entry);
			}
		}

		let cwd = match &self.cwd {
			Some(path) => PathBuf::from(path),
			None => current_dir().unwrap_or_default(),
		};

		cmd.stdin(stdin).stdout(stdout).stderr(stderr)
			.args(self.args.clone()).envs(env_vars)
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
				} ,
				StartPolicy::Manual => Child {
					process: Process::NotRunning(new_process),
					start_time: None,
					status: ChildStatus::Waiting,
				}	
			};
			match (&new_child.process, &new_child.status) {
				(_, ChildStatus::Crashed) => {},
				(Process::Running(chd), _) => info!("{} ({}): Child {} now running. [{}]",
											self.name, self.command, process_nb, chd.id()),
				(Process::NotRunning(_cmd), _) => debug!("{} ({}): Child {} loaded.",
											self.name, self.command, process_nb),
			}
			self.childs.push(new_child);
		}
	}

	pub fn kill(&mut self) {
		let pre_string = format!("{} ({}):", self.name, self.command);
		for child in &mut self.childs {
			match &mut child.process {
				Process::Running(ref mut c) => {
					match c.try_wait() {
						Ok(res) => match res {
							Some(status) => info!("{pre_string} The process has already exited [{status}]"),
							None => {
								info!("{pre_string} Sinding {} to the process {}", self.stop_signal, c.id());
								unsafe {kill(c.id() as i32, self.stop_signal as i32)};
								// I'll look into the timeout later
							}
						},
						Err(e) => error!("{pre_string} Error while trying to get child information: {e}"),
					}
				}
				Process::NotRunning(_c) => info!("{pre_string} The process was not running"),
			}
		}
	}

	pub fn status(&self, all: bool) {
		match all {
			true => println!("Program: {}\ncmd: {}\nargs: {:?}",
							self.name, self.command, self.args),
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