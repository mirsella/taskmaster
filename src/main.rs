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

use crate::config::get_config;
use config::data_type::Config;
use log::{debug, error, warn};
use std::{env::args, os::unix::process::CommandExt, process::Command};
use users::get_current_uid;

fn privilege_descalation(name: Option<&str>) -> Result<(), String> {
    if get_current_uid() != 0 {
        debug!(
            "running as non-privilaged user {}",
            users::get_current_username()
                .unwrap_or("deleted".into())
                .to_string_lossy()
        );
        return Ok(());
    }
    warn!("This program should not be run as root");
    let Some(name) = name else {
        return Err("Please specify a user to relaunch into".into());
    };
    let user =
        users::get_user_by_name(name).ok_or(format!("User {} not found on the system", name))?;
    Err(Command::new(args().next().unwrap())
        .uid(user.uid())
        .gid(user.primary_group_id())
        .exec()
        .to_string())
}

fn main() {
    // TODO: start syslog and simple_logger

    let conf_path = args().nth(1).unwrap_or("taskmaster.toml".to_string());
    let conf: Config = match get_config(conf_path.clone().into()) {
        Ok(v) => v,
        Err(e) => {
            eprintln!("Error while parsing the configuration file {conf_path:?}: {e:#?}",);
            return;
        }
    };
    if let Err(e) = privilege_descalation(conf.user.as_deref()) {
        error!("de-escalating privileges: {:#?}", e);
        return;
    }
}
