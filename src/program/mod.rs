/* ************************************************************************** */
/*                                                                            */
/*                                                        :::      ::::::::   */
/*   mod.rs                                             :+:      :+:    :+:   */
/*                                                    +:+ +:+         +:+     */
/*   By: nguiard <nguiard@student.42.fr>            +#+  +:+       +#+        */
/*                                                +#+#+#+#+#+   +#+           */
/*   Created: 2024/02/22 10:40:09 by nguiard           #+#    #+#             */
/*   Updated: 2024/02/23 15:25:11 by nguiard          ###   ########.fr       */
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
    fs::{File, OpenOptions},
    path::{Path, PathBuf},
    process::{self, Command, Stdio},
    time::{Duration, Instant},
};
use tracing::{debug, error, info, instrument, trace, warn};
use ChildStatus::*;

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

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ChildStatus {
    Stopped,
    Running,
    Waiting,
    Crashed,
	BeingKilled,
	SentSIGKILL,
	MaxRestarts,
	Unknown,
	ChildError(String),
}

impl fmt::Display for ChildStatus {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{self:?}")
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
    pub last_update: Instant,
    pub status: ChildStatus,
    pub restats: isize,
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
pub fn generate_name() -> String {
    names::Generator::default().next().unwrap()
}


impl Program {
    #[instrument(skip_all)]
    fn create_child(
        &mut self,
        cmd: &mut process::Command,
    ) -> Child {
        let setup_io = |path: Option<&Path>, file_options: &mut OpenOptions| {
            path.map_or(Ok(Stdio::null()), |path| {
                file_options
                    .open(path)
                    .map_err(|e| format!("opening file `{path:?}`: {e}"))
                    .map(Stdio::from)
            })
        };
        trace!(name = self.name, config = ?self.stdin, "Setting up stdin");
        let stdin = setup_io(self.stdin.as_deref(), File::options().read(true)).unwrap(); // CHANGE
        trace!(name = self.name, config = ?self.stdout, "Setting up stdout");
        let stdout = setup_io(
            self.stdout.as_deref(),
            File::options()
                .write(true)
                .truncate(self.stdout_truncate)
                .create(true),
        ).unwrap(); // CHANGE
        trace!(name = self.name, config = ?self.stderr, "Setting up stderr");
        let stderr = setup_io(
            self.stderr.as_deref(),
            File::options()
                .write(true)
                .truncate(self.stderr_truncate)
                .create(true),
        ).unwrap(); // CHANGE
        trace!(name = self.name, "Setting up stdio done");

        let mut env_vars = HashMap::new();
        for entry in self.env.clone() {
            let parts = entry
                .split_once('=')
                .ok_or(format!("Invalid env var: {entry}")).unwrap(); // CHANGE
            env_vars.insert(parts.0.to_string(), parts.1.to_string());
        }

        let cwd = match &self.cwd {
            Some(path) => PathBuf::from(path),
            None => current_dir().unwrap_or_default(),
        };

        match cmd
		.stdin(stdin)
		.stdout(stdout)
		.stderr(stderr)
            .args(self.args.clone())
            .envs(env_vars)
            .current_dir(cwd)
            .spawn() {
			Ok(new_child) => Child {
				process: Process::Running(new_child),
				last_update: Instant::now(),
				status: Running,
				restats: 0,
			},
			Err(e) => {
				error!("Error while creating child: {e}");
				Child {
					process: Process::NotRunning(Command::new(self.command.clone())),
					last_update: Instant::now(),
					status: Crashed,
					restats: 1,
				}
			}
		}
    }

    pub fn launch(&mut self) {
        for process_nb in 1..=self.processes {
            let mut new_process = Command::new(self.command.clone());
            let new_child = match self.start_policy {
                StartPolicy::Auto => self.create_child(&mut new_process),
                StartPolicy::Manual => Child {
                    process: Process::NotRunning(new_process),
                    last_update: Instant::now(),
                    status: Waiting,
					restats: 0,
                },
            };
            match (&new_child.process, &new_child.status) {
                (_, Crashed) => {}
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
								// Update -> timeoute gere dans update() -> try_force_kill()
                            }
                        },
                        Err(e) => {
                            error!("{pre_string} Error while trying to get child information: {e}")
                        }
                    }
                }
                Process::NotRunning(_c) => debug!("{pre_string} The process was not running"),
            }
        }
        info!("{pre_string} All children have been stopped");
    }

	/// Returns updates a supposedly running child
	/// 
	/// Some()	-> Updated child <br />
	/// None	-> Child is still running and well, no need to update
	fn update_running_child(&self, process_child: &mut process::Child, 
		child: &mut Child) -> () {
		let pid = process_child.id();
		match process_child.try_wait() {
			Ok(res) => match res {
				Some(exit_code) => {
					match exit_code.code() {
						Some(code) => {
							if self.valid_exit_codes.contains(&(code as u8)) {
								info!("Child {pid} exited successfully");
								child.status = Stopped;
							}
							else {
								info!("Child {pid} exited with a status code of {exit_code} which is unexpected");
								child.restats += 1;
								if child.restats == self.max_restarts {
									child.status = MaxRestarts;
								}
								else {
									child.status = Crashed;
								}
							}
						}
						None => {
							info!("Child {pid} has been killed by a signal");
							child.status = Crashed;
						}
					}
				}
				None => { },
			}
			Err(e) => {
				error!("Could not wait for process {}: {e}", process_child.id());
				child.status = Unknown;
			}
		};
	}

	fn try_force_kill(&self, child: &mut Child) -> () {
		if (Instant::now() - child.last_update) > self.graceful_timeout {
			match &mut child.process {
				Process::Running(c) => {
					child.last_update = Instant::now();
					child.status = SentSIGKILL;
					match &c.kill() {
						Ok(_) => { },
						Err(e) => error!("Could not kill process {}: {e}", c.id()),
					}
				}
				Process::NotRunning(_) => {
					warn!("Tried to kill a process that is not running");
				},
			}
		}
	}

	pub fn update(&mut self) {
		for i in 0..self.childs.len() {
			match &mut self.childs[i].process {
				Process::Running(ref mut c) => {
					match self.childs[i].status {
						Running => self.update_running_child(&mut c, &mut self.childs[i]),
						Unknown => {
							// Unknown == N'as pas pu avoir l'exit status dans 
							//	update_running_child(). Donc jsp si j'essaye de
							//	re-update ou si j'attend la prochaine action
							//	de l'utilisateur
						},
						BeingKilled => self.try_force_kill(&mut self.childs[i]),
						SentSIGKILL => {
							match c.try_wait() {
								Ok(Some(_)) => self.childs[i].status = Crashed,
								Ok(None) => {}, // Did not get killed yet
								Err(e) => {
									error!("Could not wait for child {}: {e}", c.id());
									self.childs[i].status = ChildError(e.to_string());
								},
							}
						},
						_ => {},
					};
				},
				Process::NotRunning(chld) => { }
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
            println!(" {:?}", Instant::now() - child.last_update);

        }
    }
}

#[cfg(test)]
mod parsing_tests {

    #[test]
    fn default() {
        
    }
}