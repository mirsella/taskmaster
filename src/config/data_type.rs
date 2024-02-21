/* ************************************************************************** */
/*                                                                            */
/*                                                        :::      ::::::::   */
/*   data_type.rs                                       :+:      :+:    :+:   */
/*                                                    +:+ +:+         +:+     */
/*   By: nguiard <nguiard@student.42.fr>            +#+  +:+       +#+        */
/*                                                +#+#+#+#+#+   +#+           */
/*   Created: 2024/02/21 13:39:16 by nguiard           #+#    #+#             */
/*   Updated: 2024/02/21 16:25:11 by nguiard          ###   ########.fr       */
/*                                                                            */
/* ************************************************************************** */

use std::time::Duration;

use crate::config::signal::Signal;
use serde::{Deserialize, Deserializer};

#[derive(Deserialize, Debug, Default, PartialEq, Eq)]
#[serde(rename_all = "lowercase")]
pub enum RestartPolicy {
    #[default]
    Never,
    Always,
    UnexpectedExit,
}
#[derive(Deserialize, Debug, Default, PartialEq, Eq)]
#[serde(rename_all = "lowercase")]
pub enum Start {
    #[default]
    Auto,
    Manual,
}

#[derive(Deserialize, Debug)]
pub struct Program {
    pub command: String,

    #[serde(default)]
    pub start: Start,

    #[serde(default = "default_processes")]
    pub processes: u8,

    #[serde(default, deserialize_with = "deserialize_duration")]
    pub min_runtime: Duration,

    #[serde(default)]
    pub exit_codes: Vec<u8>,

    #[serde(default)]
    pub restart_policy: RestartPolicy,

    pub max_restarts: u32,

    #[serde(default)]
    pub exit_signal: Signal,

    pub stdin: Option<String>,

    pub stdout: Option<String>,

    pub env: Option<Vec<String>>,

    pub cwd: Option<String>,

    pub umask: Option<String>,
}

fn default_processes() -> u8 {
    1
}

fn deserialize_duration<'de, D: Deserializer<'de>>(deserializer: D) -> Result<Duration, D::Error> {
    let secs = u64::deserialize(deserializer)?;
    Ok(Duration::from_secs(secs))
}

#[derive(Deserialize, Debug)]
pub struct Config {
    pub user: Option<String>,
    pub program: Vec<Program>,
}
