/* ************************************************************************** */
/*                                                                            */
/*                                                        :::      ::::::::   */
/*   child.rs                                           :+:      :+:    :+:   */
/*                                                    +:+ +:+         +:+     */
/*   By: nguiard <nguiard@student.42.fr>            +#+  +:+       +#+        */
/*                                                +#+#+#+#+#+   +#+           */
/*   Created: 2024/02/23 17:47:41 by nguiard           #+#    #+#             */
/*   Updated: 2024/02/27 14:36:05 by nguiard          ###   ########.fr       */
/*                                                                            */
/* ************************************************************************** */

use super::{Program, RestartPolicy};
use std::{
    error::Error,
    fmt,
    os::unix::process::ExitStatusExt,
    process::{self, ExitStatus},
    time::{Duration, Instant},
};
use tracing::{debug, error, instrument, trace, warn};

#[derive(Debug, Eq, Clone, Copy)]
pub enum Status {
    /// The process is not running
    Stopped(Instant),
    Finished(Instant, ExitStatus),
    /// being gracefully terminated
    Terminating(Instant),
    /// The process is currently starting, but before min_runtime
    Starting(Instant),
    /// after min_runtime
    Running(Instant),
    Crashed(Instant),
}

impl fmt::Display for Status {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Status::Stopped(_) => write!(f, "Stopped"),
            Status::Starting(_) => write!(f, "Starting"),
            Status::Terminating(_) => write!(f, "Terminating"),
            Status::Running(_) => write!(f, "Running"),
            Status::Finished(_, _) => write!(f, "Finished"),
            Status::Crashed(_) => write!(f, "Crashed"),
        }
    }
}

impl PartialEq for Status {
    fn eq(&self, other: &Self) -> bool {
        matches!(
            (self, other),
            (Status::Finished(_, _), Status::Finished(_, _))
                | (Status::Stopped(_), Status::Stopped(_))
                | (Status::Starting(_), Status::Starting(_))
                | (Status::Terminating(_), Status::Terminating(_))
                | (Status::Running(_), Status::Running(_))
                | (Status::Crashed(_), Status::Crashed(_))
        )
    }
}

impl Status {
    pub fn color(&self) -> ratatui::style::Color {
        match self {
            Status::Stopped(_) => ratatui::style::Color::Blue,
            Status::Starting(_) => ratatui::style::Color::Cyan,
            Status::Terminating(_) => ratatui::style::Color::Yellow,
            Status::Running(_) => ratatui::style::Color::Green,
            Status::Finished(_, _) => ratatui::style::Color::Gray,
            Status::Crashed(_) => ratatui::style::Color::Red,
        }
    }
}

#[derive(Debug)]
pub struct Child {
    pub process: process::Child,
    pub status: Status,
    pub restarts: usize,
}

impl Child {
    pub fn new(child: process::Child) -> Self {
        Child {
            process: child,
            status: Status::Starting(Instant::now()),
            restarts: 0,
        }
    }

    /// Logs the assignation of the status and assigns
    fn log_assign_status(
        &mut self,
        status: Status,
        program: &Program,
        signal: Option<i32>,
        exit: ExitStatus,
    ) {
        self.status = status;
        match self.status {
            Status::Crashed(_) => {
                debug!(
                    pid = self.process.id(),
                    "{}: child process got killed or ended unexpectedly: {}",
                    program.name,
                    signal.unwrap_or(exit.code().unwrap()),
                );
            }
            Status::Finished(_, _) => {
                debug!(
                    pid = self.process.id(),
                    name = program.name,
                    "exit code" = exit.code(),
                    "child process finished"
                );
            }
            Status::Stopped(_) => {
                debug!(
                    pid = self.process.id(),
                    name = program.name,
                    "child process stopped"
                );
            }
            Status::Terminating(_) => {
                debug!(
                    pid = self.process.id(),
                    name = program.name,
                    "child getting terminated"
                );
            }
            Status::Running(_) => {
                debug!(
                    pid = self.process.id(),
                    name = program.name,
                    "child is now running"
                );
            }
            Status::Starting(_) => {
                debug!(
                    pid = self.process.id(),
                    name = program.name,
                    "child is starting"
                );
            }
        }
    }

    pub fn tick(&mut self, program: &mut Program) -> Result<(), Box<dyn Error>> {
        let now = Instant::now();
        match self.process.try_wait() {
            Ok(Some(status)) => match (&self.status, status.signal(), status.code()) {
                (Status::Finished(_, _), None, Some(_)) => {} // Already assigned
				(Status::Finished(_, _), Some(_), None) => {}, // Already assigned
                (Status::Crashed(_), Some(_), None) => {}     // Already assigned kill
                (Status::Crashed(_), None, Some(_)) => {}     // Already assigned bad exit
                (Status::Stopped(_), Some(_), None) => {}     // Already assigned
                (_, Some(sig), None) => {
                    if program.stop_signal as u8 == sig as u8 {
                        self.log_assign_status(
                            Status::Stopped(now),
                            program,
                            status.signal(),
                            status,
                        )
                    } else {
                        self.log_assign_status(
                            Status::Finished(now, status),
                            program,
                            status.signal(),
                            status,
                        )
                    }
                }
                (_, None, Some(code)) => {
                    if !program.valid_exit_codes.contains(&code) {
                        self.log_assign_status(
                            Status::Crashed(now),
                            program,
                            status.signal(),
                            status,
                        )
                    } else {
                        self.log_assign_status(
                            Status::Finished(now, status),
                            program,
                            status.signal(),
                            status,
                        )
                    }
                }
                (_, None, None) => {}       // wierd
                (_, Some(_), Some(_)) => {} // wierd
            },
            Ok(None) => {
                if self.status == Status::Starting(now)
                    && self.last_update().elapsed() > program.min_runtime
                {
                    self.log_assign_status(
                        Status::Running(now),
                        program,
                        None,
                        ExitStatus::default(),
                    )
                }
            }
            Err(e) => {
                warn!(
                    "couldn't get the status of the child process, weird: {:?}",
                    e
                );
            }
        };
        match (self.status, &program.restart_policy,
				self.last_update().elapsed() > Duration::from_secs(1)) {
			(_, _, false) => {}
			(Status::Terminating(since), _, _) => {
				if program.graceful_timeout < since.elapsed() {
					warn!(
						pid = self.process.id(),
						name = program.name,
						"graceful shutdown timeout, killing the child"
					);
					self.kill();
				}
				if program.min_runtime < since.elapsed() {
					trace!(
						pid = self.process.id(),
						name = program.name,
						"child is now considered as running"
					);
					self.status = Status::Running(Instant::now());
				}
			}
            (Status::Finished(_, code), RestartPolicy::UnexpectedExit, true) => {
                if !program
                    .valid_exit_codes
                    .contains(&code.code().unwrap_or_default())
                    && ((self.restarts as isize) < program.max_restarts
                        || program.max_restarts == -1)
                {
                    debug!(
                        name = program.name,
                        exit_code = code.code(),
                        "restarting a finished child"
                    );
                    self.restarts += 1;
                    let child = program.create_child()?;
					self.process = child.process;
					self.status = child.status;
                }
            }
            (Status::Finished(_, code), RestartPolicy::Always, true) => {
                debug!(
                    name = program.name,
                    exit_code = code.code(),
                    "restarting a finished child"
                );
                self.restarts += 1;
                let child = program.create_child()?;
				self.process = child.process;
				self.status = child.status;
            }
			(Status::Crashed(_), RestartPolicy::UnexpectedExit, true) => {
                if (self.restarts as isize) < program.max_restarts
                        || program.max_restarts == -1
                {
                    debug!(
                        name = program.name,
                        "restarting a crashed child"
                    );
                    self.restarts += 1;
					let child = program.create_child()?;
					self.process = child.process;
					self.status = child.status;
                }
            }
            (Status::Crashed(_), RestartPolicy::Always, true) => {
                debug!(
                    name = program.name,
                    "restarting a crashed child"
                );
                self.restarts += 1;
                self.process = program.create_child()?.process;
            }
            _ => (),
        };
        Ok(())
    }
    /// Kill the child. for graceful shutdown, check stop().
    #[instrument(skip_all)]
    pub fn kill(&mut self) {
        if let Status::Running(_) | Status::Starting(_) = self.status {
            if let Err(e) = self.process.kill() {
                error!(pid = self.process.id(), error = ?e, "couldn't kill the child");
            }
            self.status = Status::Stopped(Instant::now());
        }
    }
    /// gracefully stop the child
    #[instrument(skip_all)]
    pub fn stop(&mut self, signal: i32) {
        if let Status::Running(_) | Status::Starting(_) = self.status {
            if unsafe { libc::kill(self.process.id() as i32, signal) } != 0 {
                error!(pid = self.process.id(), "couldn't send signal to the child");
            }
            self.status = Status::Terminating(Instant::now());
        }
    }

    pub fn last_update(&self) -> Instant {
        match self.status {
            Status::Finished(t, _)
            | Status::Running(t)
            | Status::Stopped(t)
            | Status::Terminating(t)
            | Status::Starting(t) => t,
            Status::Crashed(t) => t,
        }
    }
}
