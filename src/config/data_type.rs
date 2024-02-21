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
use serde::Deserialize;

#[derive(Deserialize, Debug, Default)]
pub enum RestartPolicy {
    #[default]
    Never,
    Always,
    UnexpectedExit,
}

#[derive(Deserialize, Debug)]
pub struct Program {
    pub command: String,

    #[serde(default = "default_processes")]
    pub processes: u8,

    pub min_runtime: Option<u64>,

    pub exit_codes: Option<Vec<u8>>,

    pub restart_policy: Option<RestartPolicy>,

    pub max_restarts: Option<u32>,

    #[serde(default = "default_exit_signals")]
    pub exit_signals: Vec<Signal>,

    pub stdin: Option<String>,

    pub stdout: Option<String>,

    pub env: Option<Vec<String>>,

    pub cwd: Option<String>,

    pub umask: Option<String>,
}

fn default_processes() -> u8 {
    1
}
fn default_exit_signals() -> Vec<Signal> {
    vec![Signal::SIGKILL]
}

#[derive(Deserialize, Debug)]
pub struct Config {
    pub user: Option<String>,
	pub logfile: Option<String>,
	pub loglevel: Option<String>,
    pub program: Vec<Program>,
}
