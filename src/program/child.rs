/* ************************************************************************** */
/*                                                                            */
/*                                                        :::      ::::::::   */
/*   child.rs                                           :+:      :+:    :+:   */
/*                                                    +:+ +:+         +:+     */
/*   By: nguiard <nguiard@student.42.fr>            +#+  +:+       +#+        */
/*                                                +#+#+#+#+#+   +#+           */
/*   Created: 2024/02/23 17:47:41 by nguiard           #+#    #+#             */
/*   Updated: 2024/02/27 21:20:05 by nguiard          ###   ########.fr       */
/*                                                                            */
/* ************************************************************************** */

use crate::config::Signal;

use super::{Program, RestartPolicy};
use std::{
    error::Error,
    fmt,
    os::unix::process::ExitStatusExt,
    process,
    time::{Duration, Instant},
};
use tracing::{debug, error, instrument, trace, warn};

#[derive(Debug, Clone, Copy)]
pub enum Status {
    /// The process is not running
    Stopped(Instant),
    /// the process has finished by itself, with a status code
    Finished(Instant, i32),
    /// the process has been terminated by a signal
    Terminated(Instant, i32),
    /// being gracefully terminated
    Terminating(Instant),
    /// The process is currently starting, but before min_runtime
    Starting(Instant),
    /// after min_runtime
    Running(Instant),
}
impl Status {
    pub fn get_instant(&self) -> Instant {
        match self {
            Status::Stopped(t)
            | Status::Finished(t, _)
            | Status::Terminated(t, _)
            | Status::Terminating(t)
            | Status::Starting(t)
            | Status::Running(t) => *t,
        }
    }

    pub fn is_running(&self) -> bool {
        match self {
            Status::Running(_) | Status::Starting(_) | Status::Terminating(_) => true,
            Status::Stopped(_) | Status::Finished(_, _) | Status::Terminated(_, _) => false,
        }
    }
    pub fn eq_ignore_instant(&self, other: &Self) -> bool {
        match (self, other) {
            (Self::Stopped(_), Self::Stopped(_)) => true,
            (Self::Finished(_, a), Self::Finished(_, b)) => a == b,
            (Self::Terminated(_, a), Self::Terminated(_, b)) => a == b,
            (Self::Terminating(_), Self::Terminating(_)) => true,
            (Self::Starting(_), Self::Starting(_)) => true,
            (Self::Running(_), Self::Running(_)) => true,
            _ => false,
        }
    }
}

impl fmt::Display for Status {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Status::Stopped(_) => write!(f, "Stopped"),
            Status::Starting(_) => write!(f, "Starting"),
            Status::Terminating(_) => write!(f, "Terminating"),
            Status::Running(_) => write!(f, "Running"),
            Status::Finished(_, code) => write!(f, "Finished (code: {code})"),
            Status::Terminated(_, signal) => write!(
                f,
                "Terminated (signal: {})",
                signal
                    .to_owned()
                    .try_into()
                    .map(|s: Signal| s.to_string())
                    .unwrap_or(format!("Unknown ({signal})"))
            ),
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

    fn try_wait(&mut self, program: &Program) -> Result<(), Box<dyn Error>> {
        let status = match self.process.try_wait() {
            Ok(Some(status)) if self.status.is_running() => status,
            Err(e) => {
                warn!("couldn't get the status of the child process, weird: {e:?}");
                return Err(e.into());
            }
            _ => return Ok(()),
        };
        if let Some(sig) = status.signal() {
            self.status = Status::Terminated(Instant::now(), sig);
            let signal = Signal::try_from(sig)
                .map(|s| ToString::to_string(&s))
                .unwrap_or(format!("Unkown ({sig})"));
            debug!(
                pid = self.process.id(),
                name = program.name,
                signal,
                "child process terminated by signal"
            );
        } else if let Some(code) = status.code() {
            self.status = Status::Finished(Instant::now(), code);
            debug!(
                pid = self.process.id(),
                name = program.name,
                "exit code" = code,
                "child process finished"
            );
        };
        Ok(())
    }

    pub fn tick(&mut self, program: &mut Program) -> Result<(), Box<dyn Error>> {
        self.try_wait(program)?;
        // the third match condition is for restarting; timeout of 1 second between restart, and limited to max_restarts
        match (
            self.status,
            &program.restart_policy,
            self.status.get_instant().elapsed() > Duration::from_secs(1)
                && ((self.restarts as isize) < program.max_restarts || program.max_restarts == -1),
        ) {
            (Status::Terminating(since), _, _) if since.elapsed() > program.graceful_timeout => {
                warn!(
                    pid = self.process.id(),
                    name = program.name,
                    "graceful shutdown timeout, killing the child"
                );
                self.kill();
            }
            (Status::Finished(_, code), RestartPolicy::UnexpectedExit, true)
                if !program.valid_exit_codes.contains(&code) =>
            {
                debug!(
                    name = program.name,
                    exit_code = code,
                    "restarting a finished child"
                );
                self.restarts += 1;
                let child = program.create_child()?;
                self.process = child.process;
                self.status = child.status;
            }
            (Status::Finished(_, code), RestartPolicy::Always, true) => {
                debug!(
                    name = program.name,
                    exit_code = code,
                    "restarting a finished child"
                );
                self.restarts += 1;
                let child = program.create_child()?;
                self.process = child.process;
                self.status = child.status;
            }
            (Status::Terminated(_, code), RestartPolicy::UnexpectedExit, true)
                if program.stop_signal as i32 != code =>
            {
                debug!(
                    name = program.name,
                    signal = code,
                    "restarting a terminated child"
                );
                self.restarts += 1;
                let child = program.create_child()?;
                self.process = child.process;
                self.status = child.status;
            }
            (Status::Terminated(_, code), RestartPolicy::Always, true) => {
                debug!(
                    name = program.name,
                    signal = code,
                    "restarting a terminated child"
                );
                self.restarts += 1;
                let child = program.create_child()?;
                self.process = child.process;
                self.status = child.status;
            }
            (Status::Starting(since), _, _) if since.elapsed() > program.min_runtime => {
                self.status = Status::Running(since);
                trace!(name = program.name, "child is now considered as running");
            }
            _ => (),
        };
        Ok(())
    }

    /// Kill the child. for graceful shutdown, check stop().
    #[instrument(skip_all)]
    pub fn kill(&mut self) {
        if let Status::Running(_) | Status::Starting(_) | Status::Terminating(_) = self.status {
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
}
