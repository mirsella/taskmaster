/* ************************************************************************** */
/*                                                                            */
/*                                                        :::      ::::::::   */
/*   data_type.rs                                       :+:      :+:    :+:   */
/*                                                    +:+ +:+         +:+     */
/*   By: nguiard <nguiard@student.42.fr>            +#+  +:+       +#+        */
/*                                                +#+#+#+#+#+   +#+           */
/*   Created: 2024/02/21 13:39:16 by nguiard           #+#    #+#             */
/*   Updated: 2024/02/21 17:06:11 by nguiard          ###   ########.fr       */
/*                                                                            */
/* ************************************************************************** */

use crate::config::signal::Signal;
use libc::pid_t;
use serde::Deserialize;
use serde_with::{serde_as, DisplayFromStr, DurationSeconds};
use std::{
    path::PathBuf,
    time::{Duration, Instant},
};
use tracing::Level;

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
pub struct Child {
    pub pid: pid_t,
    pub start_time: Instant,
}

#[serde_as]
#[derive(Deserialize, Debug)]
pub struct Program {
    pub command: String,
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
    pub max_restarts: u32,
    #[serde(default)]
    pub valid_signal: Signal,
    #[serde(default)]
    #[serde_as(as = "DurationSeconds<u64>")]
    pub graceful_timeout: Duration,
    pub stdin: Option<PathBuf>,
    pub stdout: Option<PathBuf>,
    pub env: Option<Vec<String>>,
    pub cwd: Option<PathBuf>,
    pub umask: Option<String>,

    // runtime only
    #[serde(skip)]
    pub pids: Vec<Child>,
}

fn default_processes() -> u8 {
    1
}

#[serde_as]
#[derive(Deserialize, Debug)]
pub struct Config {
    pub user: Option<String>,
    #[serde(default = "default_logfile")]
    pub logfile: String,
    #[serde(default = "default_loglevel")]
    #[serde_as(as = "DisplayFromStr")]
    pub loglevel: Level,
    pub program: Vec<Program>,
}
fn default_logfile() -> String {
    "taskmaster.log".to_string()
}
fn default_loglevel() -> Level {
    Level::INFO
}
