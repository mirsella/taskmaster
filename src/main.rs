/* ************************************************************************** */
/*                                                                            */
/*                                                        :::      ::::::::   */
/*   main.rs                                            :+:      :+:    :+:   */
/*                                                    +:+ +:+         +:+     */
/*   By: nguiard <nguiard@student.42.fr>            +#+  +:+       +#+        */
/*                                                +#+#+#+#+#+   +#+           */
/*   Created: 2024/02/21 10:09:10 by nguiard           #+#    #+#             */
/*   Updated: 2024/02/21 16:14:57 by nguiard          ###   ########.fr       */
/*                                                                            */
/* ************************************************************************** */

mod config;

use config::data_type::Config;
use crate::config::get_config;
use log::{debug, error, warn};
use std::{env::{self, args}, error::Error, os::unix::process::CommandExt, path::Path, process::Command};
use users::get_current_uid;

fn privilege_descalation(user: &str) -> Result<(), String> {
    if get_current_uid() != 0 {
        debug!(
            "running as non-privilaged user {}",
            users::get_current_username()
                .unwrap_or("deleted".into())
                .to_string_lossy()
        );
        return Ok(());
    }
    warn!("This program should not be run as root. relaunching as {user}");
    let user = users::get_user_by_name(user).ok_or(format!("User {} not found", user))?;
    Err(Command::new(args().next().unwrap())
        .uid(user.uid())
        .gid(user.primary_group_id())
        .exec()
        .to_string())
}

fn main() {
    // TODO: start syslog and simple_logger
    let location: Vec<String> = env::args().collect();

	let conf_file: &Path = match location.len() {
		1 => Path::new("taskmaster.toml"),
		_ => Path::new(&location[1])
	};

	let conf: Config = match get_config(conf_file) {
		Ok(v) => v,
		Err(e) => {
			eprintln!("Error while parsing the configuration file {:?}: {:#?}",
						conf_file, e);
			return ;
		}
	};
    if let Err(e) = privilege_descalation(&conf.user) {
        error!("descalating privileges: {:#?}", e);
        return;
    }
}
