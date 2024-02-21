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

use std::path::Path;

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
pub struct Global {
	userid: Option<u32>,
}

#[derive(Deserialize, Debug)]
pub struct Program {
	command: String,

	#[serde(default = "default_processes")]
	processes: u8,
	
	#[serde(default = "default_min_runtime")]
	min_runtime: Option<u64>,
	
	#[serde(default = "default_exit_codes")]
	exit_codes: Vec<u8>,
	
	#[serde(default = "default_restart_policy")]
	restart_policy: RestartPolicy,

	#[serde(default = "default_max_restarts")]
	max_restarts: u32,

	#[serde(default = "default_exit_signals")]
	exit_signals: Vec<Signal>,

	#[serde(default = "default_io")]
	stdin: Option<String>,

	#[serde(default = "default_io")]
	stdout: Option<String>,

	#[serde(default = "default_env")]
	env: Option<Vec<String>>,

	#[serde(default = "default_cwd")]
	cwd: Option<String>,

	#[serde(default = "default_umask")]
	umask: Option<String>,
}

fn default_processes() -> u8 { 1 }
fn default_min_runtime() -> Option<u64> { None }
fn default_exit_codes() -> Vec<u8> { vec![0] }
fn default_restart_policy() -> RestartPolicy { RestartPolicy::Never }
fn default_max_restarts() -> u32 { 0 }
fn default_exit_signals() -> Vec<Signal> { vec![Signal::SIGKILL] }
fn default_io() -> Option<String> { None }
fn default_env() -> Option<Vec<String>> { None }
fn default_cwd() -> Option<String> { None }
fn default_umask() -> Option<String> { None }

#[derive(Deserialize, Debug)]
pub struct Config {
	global: Global,
	program: Vec<Program>,
}
