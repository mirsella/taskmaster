/* ************************************************************************** */
/*                                                                            */
/*                                                        :::      ::::::::   */
/*   child.rs                                           :+:      :+:    :+:   */
/*                                                    +:+ +:+         +:+     */
/*   By: nguiard <nguiard@student.42.fr>            +#+  +:+       +#+        */
/*                                                +#+#+#+#+#+   +#+           */
/*   Created: 2024/02/23 17:47:41 by nguiard           #+#    #+#             */
/*   Updated: 2024/02/23 18:57:26 by nguiard          ###   ########.fr       */
/*                                                                            */
/* ************************************************************************** */

use std::{fmt, process, time::Instant};

#[derive(Debug, Clone, PartialEq, Eq, Copy)]
pub enum ChildStatus {
    Stopped,
    Running,
    Waiting,
    Crashed,
    BeingKilled,
    SentSIGKILL,
    ShouldRestart,
    MaxRestarts,
    Unknown,
    ChildError,
}

#[derive(Debug)]
pub struct Child {
    pub process: Option<process::Child>,
    pub last_update: Instant,
    pub status: ChildStatus,
    pub restarts: isize,
}

impl fmt::Display for ChildStatus {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{self:?}")
    }
}

impl Child {
    pub fn new(process: process::Child) -> Self {
        Child {
            process: Some(process),
            last_update: Instant::now(),
            status: ChildStatus::Running,
            restarts: 0,
        }
    }

    pub fn my_take(&mut self) -> Self {
        Child {
            process: self.process.take(),
            last_update: self.last_update,
            status: self.status,
            restarts: self.restarts,
        }
    }

    pub fn check_running(&mut self) -> Result<(), String> {
        match &mut self.process {
            Some(c) => match c.try_wait() {
                Ok(Some(code)) => Err(format!("{} already exited with status code {code}", c.id())),
                Ok(None) => Ok(()),
                Err(e) => Err(e.to_string()),
            },
            None => Err("Process is not running".to_string()),
        }
    }

    pub fn restart(&mut self, max_restarts: isize) {
        match (&self.process, &self.status, self.restarts >= max_restarts) {
            (Some(_), ChildStatus::ShouldRestart, _) => {} // Dont restart
            (Some(_), _, _) => {}                          // Dont restart
            (None, ChildStatus::MaxRestarts, _) => {}
            (None, _, _) => {} // Restart
        }
    }
}
