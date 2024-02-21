/* ************************************************************************** */
/*                                                                            */
/*                                                        :::      ::::::::   */
/*   data_type.rs                                       :+:      :+:    :+:   */
/*                                                    +:+ +:+         +:+     */
/*   By: nguiard <nguiard@student.42.fr>            +#+  +:+       +#+        */
/*                                                +#+#+#+#+#+   +#+           */
/*   Created: 2024/02/21 13:39:16 by nguiard           #+#    #+#             */
/*   Updated: 2024/02/21 15:10:06 by nguiard          ###   ########.fr       */
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

    pub exit_codes: Vec<u8>,

    pub restart_policy: RestartPolicy,

    pub max_restarts: u32,

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
    pub user: String,
    pub program: Vec<Program>,
}
